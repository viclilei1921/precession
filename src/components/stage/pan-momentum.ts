/**
 * 放大后双指平移的惯性滑行。
 * 手感对齐 docs/viewer-gestures.md 方案：
 * - momentum 滚轮事件到来时采样速度；真实滚轮/指针按下立刻掐灭。
 * - requestAnimationFrame 衰减滑行（0.92/帧，60fps 基准约 350ms 收敛），
 *   速度封顶，防一次猛滑飞出几屏。
 * - 位移由外部用 clampPan 收边；撞边把速度清零（硬停，不回弹、
 *   不把残余溢出成翻页）。
 * - prefers-reduced-motion 时不启用惯性，位移立即停。
 * 旧内核没有 momentum 标记时，这个模块不会被喂到事件，平移退化为无惯性。
 */

const FRICTION = 0.92;
const MAX_SPEED = 2400; // px/s
const STILL = 24; // px/s 以下视为停止

export type PanMomentum = {
  /** momentum 滚轮事件采样。delta 为该次事件的位移增量（px）。 */
  feed(dx: number, dy: number, dtMs: number): void;
  /** 真实滚轮事件或指针按下：立刻停掉惯性。 */
  stop(): void;
  /** 位移被 clampPan 收边（撞到图片边界）时清掉速度。 */
  halt(): void;
  dispose(): void;
};

function prefersReducedMotion() {
  return typeof window.matchMedia === 'function' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

export function createPanMomentum(apply: (dx: number, dy: number) => void): PanMomentum {
  let vx = 0; // px/s
  let vy = 0; // px/s
  let raf = 0;
  let last = 0;
  let disposed = false;

  const cancel = () => {
    if (raf !== 0) {
      window.cancelAnimationFrame(raf);
      raf = 0;
    }
    last = 0;
  };

  const tick = (now: number) => {
    raf = 0;
    if (disposed) {
      return;
    }
    const dt = last === 0 ? 0 : Math.min(0.05, (now - last) / 1000);
    last = now;
    if (dt <= 0) {
      schedule();
      return;
    }
    const dx = vx * dt;
    const dy = vy * dt;
    vx *= FRICTION;
    vy *= FRICTION;
    if (Math.hypot(dx, dy) > 0.05) {
      apply(dx, dy);
    }
    if (Math.abs(vx) < STILL && Math.abs(vy) < STILL) {
      cancel();
      vx = 0;
      vy = 0;
      return;
    }
    schedule();
  };

  const schedule = () => {
    if (raf === 0 && !disposed) {
      raf = window.requestAnimationFrame(tick);
    }
  };

  return {
    feed(dx, dy, dtMs) {
      if (disposed || prefersReducedMotion()) {
        return;
      }
      const dt = Math.max(8, Math.min(100, dtMs)) / 1000;
      // 惯性尾巴的相邻事件差分出速度，封顶防猛滑
      const sx = Math.max(-MAX_SPEED, Math.min(MAX_SPEED, dx / dt));
      const sy = Math.max(-MAX_SPEED, Math.min(MAX_SPEED, dy / dt));
      // 尾部事件渐弱，速度取当前与采样的较大值，滑行只衰减
      vx = Math.sign(sx) * Math.max(Math.abs(vx), Math.abs(sx));
      vy = Math.sign(sy) * Math.max(Math.abs(vy), Math.abs(sy));
      if (last === 0) {
        last = performance.now();
      }
      schedule();
    },
    stop() {
      vx = 0;
      vy = 0;
      cancel();
    },
    halt() {
      vx = 0;
      vy = 0;
      cancel();
    },
    dispose() {
      disposed = true;
      cancel();
    }
  };
}
