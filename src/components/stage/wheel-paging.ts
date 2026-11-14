/**
 * 缩放不超过适应窗口时的纵向翻页条。
 * 滚轮和拖动先带动上一张、当前张、下一张。滚动停下再决定：
 * 走过四分之一屏，或者甩得够快，就播完动画翻一页；否则弹回。
 * 翻页动画把目标图送到正中。这段动画和随后的惯性不再计入下一张。
 */
const IDLE_MS = 80;
const SETTLE_MS = 280;
const COMMIT_RATIO = 0.25;
const VELOCITY = 0.9;
const RUBBER_RATIO = 0.25;
const EDGE_RATIO = 1.4;

/** 翻页会换掉舞台。惯性尾巴留在这里，新舞台才不会接着被带着走。 */
const tail = {
  spent: false,
  timer: 0
};

function swallowTail() {
  tail.spent = true;
  window.clearTimeout(tail.timer);
  tail.timer = window.setTimeout(() => {
    tail.spent = false;
  }, IDLE_MS);
}

function releaseTail() {
  tail.spent = false;
  window.clearTimeout(tail.timer);
}

export type StripPager = {
  /** 滚轮。deltaY 向下为正，画面往上走，朝下一张。 */
  moveWheel(deltaY: number, height: number): void;
  /** 指针下移为正，画面跟着下移，朝上一张。 */
  beginPointer(): boolean;
  movePointer(deltaY: number, height: number): void;
  release(height: number): void;
  reset(): void;
  dispose(): void;
};

export function createStripPager(options: {
  onOffset: (offset: number) => void;
  canStep: (direction: 1 | -1) => boolean;
  onCommit: (direction: 1 | -1) => void;
  onArm?: () => void;
}): StripPager {
  let offset = 0;
  let raf = 0;
  let idle = 0;
  let pointers = 0;
  let disposed = false;
  let phase: 'idle' | 'track' | 'settle' = 'idle';
  let settlingCommit = false;
  const samples: { t: number; offset: number }[] = [];

  const publish = (value: number) => {
    if (value === offset) {
      return;
    }
    offset = value;
    if (!disposed) {
      options.onOffset(value);
    }
  };

  const cancelAnimation = () => {
    if (raf !== 0) {
      cancelAnimationFrame(raf);
      raf = 0;
    }
  };

  const clearIdle = () => {
    window.clearTimeout(idle);
    idle = 0;
  };

  const noteSample = () => {
    const now = performance.now();
    samples.push({ t: now, offset });
    const cutoff = now - IDLE_MS;
    while (samples.length > 1 && samples[0].t < cutoff) {
      samples.shift();
    }
  };

  const readVelocity = () => {
    if (samples.length < 2) {
      return 0;
    }
    const first = samples[0];
    const last = samples[samples.length - 1];
    const dt = last.t - first.t;
    if (dt < 8) {
      return 0;
    }
    return (last.offset - first.offset) / dt;
  };

  const arm = () => {
    if (phase === 'idle') {
      phase = 'track';
      samples.length = 0;
      options.onArm?.();
    }
  };

  const push = (delta: number, height: number) => {
    const proposed = offset + delta;
    const toward: 1 | -1 = proposed < 0 ? 1 : -1;
    const crossingBack = offset !== 0 && Math.sign(proposed) !== Math.sign(offset);
    if (proposed === 0 || crossingBack || Math.abs(proposed) < Math.abs(offset)) {
      return proposed;
    }
    if (!options.canStep(toward)) {
      const cap = height * RUBBER_RATIO;
      const next = offset + delta * 0.35;
      return Math.max(-cap, Math.min(cap, next));
    }
    const magnitude = Math.abs(proposed);
    if (magnitude <= height) {
      return proposed;
    }
    const limited = height + (magnitude - height) * 0.28;
    return Math.sign(proposed) * Math.min(height * EDGE_RATIO, limited);
  };

  const animateTo = (target: number, commit: 1 | -1 | 0) => {
    clearIdle();
    cancelAnimation();
    phase = 'settle';
    settlingCommit = commit !== 0;
    if (settlingCommit) {
      swallowTail();
    }
    const start = offset;
    const reduce =
      typeof window.matchMedia === 'function' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    const finish = () => {
      raf = 0;
      phase = 'idle';
      settlingCommit = false;
      if (commit !== 0 && !disposed) {
        publish(0);
        options.onCommit(commit);
        return;
      }
      publish(0);
    };
    if (reduce || start === target) {
      publish(target);
      finish();
      return;
    }
    const began = performance.now();
    const tick = (now: number) => {
      if (disposed) {
        return;
      }
      const t = Math.min(1, (now - began) / SETTLE_MS);
      const eased = 1 - (1 - t) ** 3;
      publish(start + (target - start) * eased);
      if (t < 1) {
        raf = requestAnimationFrame(tick);
        return;
      }
      finish();
    };
    raf = requestAnimationFrame(tick);
  };

  const decide = (height: number) => {
    if (phase === 'settle' || height <= 0) {
      return;
    }
    const velocity = readVelocity();
    const flicked: 1 | -1 | 0 = Math.abs(velocity) >= VELOCITY ? (velocity < 0 ? 1 : -1) : 0;
    const positional: 1 | -1 | 0 = Math.abs(offset) > height * COMMIT_RATIO ? (offset < 0 ? 1 : -1) : 0;
    const direction = flicked !== 0 ? flicked : positional;
    samples.length = 0;
    if (direction !== 0 && options.canStep(direction)) {
      animateTo(direction > 0 ? -height : height, direction);
      return;
    }
    animateTo(0, 0);
  };

  const scheduleIdle = (height: number) => {
    if (pointers > 0) {
      return;
    }
    clearIdle();
    idle = window.setTimeout(() => {
      idle = 0;
      decide(height);
    }, IDLE_MS);
  };

  return {
    moveWheel(deltaY: number, height: number) {
      if (disposed || deltaY === 0 || height <= 0) {
        return;
      }
      if (phase === 'settle') {
        if (settlingCommit) {
          swallowTail();
        }
        return;
      }
      if (tail.spent) {
        swallowTail();
        return;
      }
      arm();
      noteSample();
      publish(push(-deltaY, height));
      noteSample();
      scheduleIdle(height);
    },
    beginPointer() {
      if (disposed || phase === 'settle') {
        return false;
      }
      releaseTail();
      pointers += 1;
      clearIdle();
      return true;
    },
    movePointer(deltaY: number, height: number) {
      if (disposed || pointers === 0 || phase === 'settle' || height <= 0 || deltaY === 0) {
        return;
      }
      arm();
      clearIdle();
      noteSample();
      publish(push(deltaY, height));
      noteSample();
    },
    release(height: number) {
      if (pointers > 0) {
        pointers -= 1;
      }
      if (disposed || pointers > 0 || phase !== 'track') {
        return;
      }
      decide(height);
    },
    reset() {
      cancelAnimation();
      clearIdle();
      pointers = 0;
      phase = 'idle';
      settlingCommit = false;
      samples.length = 0;
      publish(0);
    },
    dispose() {
      disposed = true;
      cancelAnimation();
      clearIdle();
    }
  };
}
