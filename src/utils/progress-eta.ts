/** 用进度增量估算剩余秒数。进度不变时不更新速度，避免原地踏步把速度算成 0。 */
export class ProgressETA {
  private smoothing: number;
  private lastTime: number;
  private lastProgress = 0;
  private averageSpeed = 0;

  constructor(smoothing = 0.2) {
    this.smoothing = smoothing;
    this.lastTime = performance.now();
  }

  /** 进度为 0 到 100。还没有速度时返回 null，完成时返回 0。 */
  update(progress: number): number | null {
    if (progress <= 0) {
      return null;
    }
    if (progress >= 100) {
      return 0;
    }

    const now = performance.now();
    const timeDiff = now - this.lastTime;
    const progressDiff = progress - this.lastProgress;

    if (progressDiff > 0 && timeDiff > 0) {
      const currentSpeed = progressDiff / timeDiff;
      this.averageSpeed =
        this.averageSpeed === 0
          ? currentSpeed
          : this.smoothing * currentSpeed + (1 - this.smoothing) * this.averageSpeed;
      this.lastTime = now;
      this.lastProgress = progress;
    }

    if (this.averageSpeed === 0) {
      return null;
    }

    return (100 - progress) / this.averageSpeed / 1000;
  }

  reset() {
    this.lastTime = performance.now();
    this.lastProgress = 0;
    this.averageSpeed = 0;
  }
}

/** 剩余时间显示为 `m:ss`。 */
export function formatRemaining(seconds: number) {
  const total = Math.max(0, Math.round(seconds));
  const minutes = Math.floor(total / 60);
  const remain = total % 60;
  return `${minutes}:${String(remain).padStart(2, '0')}`;
}
