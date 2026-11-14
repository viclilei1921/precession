/**
 * 缩放不超过适应窗口时，滚轮先带动整张图上下移动，超过阈值再翻页。
 * 翻页后视口先回到正中。同一次滚动还没停的事件不再计入下一张，
 * 否则惯性会连着翻很多张。停在阈值以内就回弹。
 */
const RELEASE_MS = 300;
const SETTLE_MS = 240;

/** 翻页会换掉 Stage。这段状态留在模块上，新舞台才能接着把视口按在正中。 */
const gesture = {
  held: false,
  timer: 0
};

function holdViewport() {
  gesture.held = true;
  window.clearTimeout(gesture.timer);
  gesture.timer = window.setTimeout(() => {
    gesture.held = false;
  }, RELEASE_MS);
}

/** 翻页阈值。画面要移过视口高度的 60% 才换页。 */
const PULL_RATIO = 0.6;

export function pullThreshold(viewHeight: number) {
  const height = viewHeight > 0 ? viewHeight : 640;
  return Math.round(height * PULL_RATIO);
}

export type WheelPull = {
  /** deltaY 向下为正。画面往相反方向走，越过阈值时 step。step 返回 false 表示这个方向没有图。 */
  move(deltaY: number, threshold: number): void;
  reset(): void;
  dispose(): void;
};

export function createWheelPull(options: {
  onChange: (offset: number) => void;
  step: (direction: 1 | -1) => boolean;
}): WheelPull {
  let offset = 0;
  let raf = 0;
  let idle = 0;
  let disposed = false;

  const publish = (value: number) => {
    if (value === offset) {
      return;
    }
    offset = value;
    if (!disposed) {
      options.onChange(value);
    }
  };

  const cancelSpring = () => {
    if (raf !== 0) {
      cancelAnimationFrame(raf);
      raf = 0;
    }
  };

  const spring = () => {
    cancelSpring();
    const start = offset;
    if (start === 0) {
      return;
    }
    const reduce =
      typeof window.matchMedia === 'function' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    if (reduce) {
      publish(0);
      return;
    }
    const began = performance.now();
    const tick = (now: number) => {
      if (disposed) {
        return;
      }
      const t = Math.min(1, (now - began) / SETTLE_MS);
      const eased = 1 - (1 - t) ** 3;
      publish(start * (1 - eased));
      if (t < 1) {
        raf = requestAnimationFrame(tick);
      } else {
        raf = 0;
        publish(0);
      }
    };
    raf = requestAnimationFrame(tick);
  };

  const scheduleRelease = () => {
    window.clearTimeout(idle);
    idle = window.setTimeout(() => {
      if (!disposed && offset !== 0) {
        spring();
      }
    }, RELEASE_MS);
  };

  return {
    move(deltaY: number, threshold: number) {
      if (disposed || deltaY === 0 || threshold <= 0) {
        return;
      }
      if (gesture.held) {
        cancelSpring();
        window.clearTimeout(idle);
        publish(0);
        holdViewport();
        return;
      }
      cancelSpring();
      const direction: 1 | -1 = deltaY > 0 ? 1 : -1;
      const next = offset - deltaY;
      if (Math.abs(next) <= threshold) {
        publish(next);
        scheduleRelease();
        return;
      }
      if (options.step(direction)) {
        window.clearTimeout(idle);
        publish(0);
        holdViewport();
        return;
      }
      publish((direction > 0 ? -1 : 1) * threshold);
      scheduleRelease();
    },
    reset() {
      cancelSpring();
      window.clearTimeout(idle);
      publish(0);
    },
    dispose() {
      disposed = true;
      cancelSpring();
      window.clearTimeout(idle);
    }
  };
}
