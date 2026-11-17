/**
 * 缩放不超过适应窗口时的纵向翻页条。
 * 手感对齐 docs/图片查看器手势实验台：
 * - 带 `WheelEvent.momentum` 标记的惯性事件全部丢弃，画面不动；惯性到来即结算手势。
 *   内核不支持 momentum 时退化为静默判定：120ms 没有新滚轮事件视为手势结束。
 * - 静默结算时长区分内核：有 momentum 标记的内核放宽到 600ms——抬手带惯性会立刻
 *   收到 momentum 事件即时结算，慢路径只兜轻抬无惯性的情形，手指停在触摸板上
 *   不动不会被误判成松手。不支持 momentum 的内核维持 120ms，静默即松手。
 * - 亚像素抖动攒满 1px 才推进：手指停在触摸板上时 PTP 驱动仍会送出正负相抵的
 *   小数 delta，直接累加会慢慢爬过翻页阈值，进位过滤后永远凑不满 1px。
 * - 位移跟手推进：触摸板 1:1，鼠标滚轮 100ms 平滑；超过 50% 页高后阻尼递增，
 *   一次手势最多推到 70% 页高，推不动。
 * - 手势结束时结算：位移达到 50% 页高翻一页（500ms 动画把邻图送到正中），
 *   否则回弹（320ms）。翻页/回弹期间滚动全部拒绝，一段手势最多翻一页。
 * - 已经在首尾、推不出去的方向用小阻尼橡皮筋兜住，抬手弹回。
 * 翻页会换掉舞台。惯性尾巴留在这里，新舞台才不会接着被带着走。
 */

const TRIGGER_RATIO = 0.5;
const DAMP_RATIO = 0.5;
const MAX_RATIO = 0.7;
const FLIP_MS = 500;
const SNAP_MS = 320;
const SMOOTH_MS = 100;
const SILENCE_MS = 120;
/** 支持 momentum 标记的内核上的静默结算时长：慢路径只兜轻抬无惯性 */
const SILENCE_MS_SLOW = 600;
const TAIL_MS = 100;
const RUBBER_RATIO = 0.2;
const RUBBER_GAIN = 0.3;

export const PAGING_EASE = 'cubic-bezier(.32,.72,.24,1)';

/** 翻页动画结束后短暂丢弃滚动输入，兜住没有 momentum 标记的惯性尾巴。 */
const tail = {
  spent: false,
  timer: 0
};

function swallowTail() {
  tail.spent = true;
  window.clearTimeout(tail.timer);
  tail.timer = window.setTimeout(() => {
    tail.spent = false;
  }, TAIL_MS);
}

function prefersReducedMotion() {
  return typeof window.matchMedia === 'function' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

export type StripPager = {
  /**
   * 滚轮。delta 是换算成容器位移的增量：向下为负（朝下一张），向上为正。
   * momentum 为 true 的事件不进位移，只负责结算未完的手势。
   */
  moveWheel(delta: number, height: number, input: { momentum: boolean; smooth: boolean }): void;
  /**
   * 真实手势推进容器（指针拖动、放大时图片吃剩的残余位移）。
   * 指针还按着时不结算，松手走 release；纯滚轮残余则走静默判定。
   */
  push(delta: number, height: number, smooth: boolean): void;
  /** 指针开始带动翻页条。翻页/回弹动画中返回 false。 */
  beginPointer(): boolean;
  /** 手指松开，立刻结算这一次手势。 */
  release(height: number): void;
  /**
   * 触摸板手指接触状态（Rust 侧 Raw Input 的确定性信号）。
   * onBoard=true：手指在板上，撤掉静默计时，永不误结算；
   * onBoard=false：手指已离开，立即结算手势（0ms，替代静默猜测）。
   */
  setTouchContact(onBoard: boolean): void;
  reset(): void;
  dispose(): void;
};

export function createStripPager(options: {
  onOffset: (offset: number, animateMs: number) => void;
  canStep: (direction: 1 | -1) => boolean;
  onCommit: (direction: 1 | -1) => void;
  onArm?: () => void;
}): StripPager {
  /** 当前渲染位移：>0 展示上一张，<0 展示下一张 */
  let offset = 0;
  /** 手势意图位移：滚动/拖动累加出的目标 */
  let target = 0;
  let height = 0;
  let pointers = 0;
  let disposed = false;
  let phase: 'idle' | 'animating' = 'idle';
  let gestureOpen = false;
  let silence = 0;
  let settle = 0;
  /** 是否见过 momentum 标记（新内核）。决定静默结算走 600ms 慢路径还是 120ms 快路径 */
  let seenMomentum = false;
  /** 亚像素抖动进位过滤：小数 delta 在这里攒，攒满 1px 才吐给推进逻辑 */
  let subpixel = 0;
  /** 触摸板手指在板上（Rust Raw Input 确定性信号）。true 时静默计时撤掉 */
  let touchOnBoard = false;

  const publish = (value: number, animateMs = 0) => {
    offset = value;
    if (!disposed) {
      options.onOffset(value, animateMs);
    }
  };

  const clearSilence = () => {
    window.clearTimeout(silence);
    silence = 0;
  };

  const clearSettle = () => {
    window.clearTimeout(settle);
    settle = 0;
  };

  const armSilence = () => {
    clearSilence();
    // 手指还在板上（Raw Input 信号）时不武装静默计时，等离板信号结算
    if (touchOnBoard) {
      return;
    }
    silence = window.setTimeout(
      () => {
        silence = 0;
        endGesture();
      },
      seenMomentum ? SILENCE_MS_SLOW : SILENCE_MS
    );
  };

  function snapBack() {
    clearSilence();
    clearSettle();
    if (Math.abs(offset) < 1 && Math.abs(target) < 1) {
      target = 0;
      publish(0);
      return;
    }
    const ms = prefersReducedMotion() ? 0 : SNAP_MS;
    target = 0;
    phase = 'animating';
    settle = window.setTimeout(() => {
      settle = 0;
      phase = 'idle';
      publish(0, 0);
    }, ms);
    publish(0, ms);
  }

  function goAnimated(direction: 1 | -1) {
    clearSilence();
    clearSettle();
    if (!options.canStep(direction)) {
      snapBack();
      return;
    }
    const ms = prefersReducedMotion() ? 0 : FLIP_MS;
    phase = 'animating';
    swallowTail();
    settle = window.setTimeout(() => {
      settle = 0;
      phase = 'idle';
      target = 0;
      publish(0, 0);
      swallowTail();
      if (!disposed) {
        options.onCommit(direction);
      }
    }, ms);
    target = direction > 0 ? -height : height;
    publish(target, ms);
  }

  /** 手势结束：按意图位移结算，一段手势最多翻一页。 */
  function endGesture() {
    clearSilence();
    gestureOpen = false;
    if (phase === 'animating') {
      return;
    }
    const abs = Math.abs(target);
    if (height > 0 && abs >= height * TRIGGER_RATIO) {
      goAnimated(target > 0 ? -1 : 1);
      return;
    }
    if (abs > 2) {
      snapBack();
      return;
    }
    target = 0;
    publish(0);
  }

  function advance(step: number, smooth: boolean) {
    if (!gestureOpen) {
      gestureOpen = true;
      options.onArm?.();
    }
    // 亚像素抖动进位：小数 delta 攒满 1px 才推进。手指停在触摸板上时
    // 驱动送来的正负相抵抖动永远凑不满 1px，不会爬过翻页阈值。
    const whole = Math.trunc(subpixel + step);
    subpixel = subpixel + step - whole;
    if (whole === 0) {
      return;
    }
    const signed = whole;
    const before = target;
    const proposed = before + signed;
    const toward: 1 | -1 = proposed < 0 ? 1 : -1;
    let next = proposed;
    if (Math.abs(proposed) > Math.abs(before)) {
      if (!options.canStep(toward)) {
        // 推不出去的方向：小阻尼橡皮筋，抬手必定弹回
        const cap = height * RUBBER_RATIO;
        next = Math.max(-cap, Math.min(cap, before + signed * RUBBER_GAIN));
      } else {
        // 50% 页高起阻尼递增，最多推到 70%
        let gain = 1;
        const start = Math.abs(before);
        if (start >= height * DAMP_RATIO) {
          const t = Math.min(1, (start - height * DAMP_RATIO) / (height * (MAX_RATIO - DAMP_RATIO)));
          gain = (1 - t) ** 1.6;
        }
        const limit = height * MAX_RATIO;
        next = Math.max(-limit, Math.min(limit, before + signed * gain));
      }
    }
    target = next;
    publish(next, smooth ? SMOOTH_MS : 0);
  }

  return {
    moveWheel(delta, moveHeight, input) {
      if (disposed) {
        return;
      }
      height = moveHeight;
      if (input.momentum) {
        // 惯性事件：画面不动；手指已离开，立刻结算这次手势
        seenMomentum = true;
        subpixel = 0;
        if (gestureOpen) {
          endGesture();
        }
        return;
      }
      if (tail.spent || phase === 'animating') {
        swallowTail();
        return;
      }
      if (moveHeight > 0 && delta !== 0) {
        advance(delta, input.smooth);
      }
      armSilence();
    },
    push(delta, pushHeight, smooth) {
      if (disposed || phase === 'animating' || pushHeight <= 0 || delta === 0) {
        return;
      }
      height = pushHeight;
      advance(delta, smooth);
      if (pointers === 0) {
        armSilence();
      }
    },
    beginPointer() {
      if (disposed || phase === 'animating') {
        return false;
      }
      clearSilence();
      pointers += 1;
      return true;
    },
    release(releaseHeight) {
      if (pointers > 0) {
        pointers -= 1;
      }
      if (disposed || pointers > 0) {
        return;
      }
      height = releaseHeight;
      endGesture();
    },
    setTouchContact(onBoard) {
      if (onBoard) {
        touchOnBoard = true;
        clearSilence();
        return;
      }
      // 手指离板：撤销暂停状态并立刻结算（没有手势时是无操作）
      touchOnBoard = false;
      if (disposed || phase === 'animating') {
        return;
      }
      if (gestureOpen && pointers === 0) {
        endGesture();
      }
    },
    reset() {
      clearSilence();
      clearSettle();
      pointers = 0;
      phase = 'idle';
      gestureOpen = false;
      target = 0;
      subpixel = 0;
      touchOnBoard = false;
      publish(0);
    },
    dispose() {
      disposed = true;
      clearSilence();
      clearSettle();
    }
  };
}
