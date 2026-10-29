const at = new Map<string, number>();

/** 小窗拖过的位置。大预览从这里接着看。 */
export function readPreviewTime(id: string) {
  return at.get(id);
}

export function writePreviewTime(id: string, time: number) {
  if (Number.isFinite(time) && time >= 0) {
    at.set(id, time);
  }
}

/** 没有记过进度时，从开头稍后再取一帧，避开纯黑的第一帧。 */
export function cueTime(duration: number, stored: number | undefined) {
  if (!Number.isFinite(duration) || duration <= 0) {
    return 0;
  }
  const seed = stored ?? 0.12;
  const limit = Math.max(0, duration - 0.04);
  return Math.min(Math.max(0, seed), limit);
}

export function bufferedEnd(video: HTMLVideoElement) {
  const ranges = video.buffered;
  if (ranges.length === 0) {
    return 0;
  }
  return ranges.end(ranges.length - 1);
}

export function clock(total: number) {
  if (!Number.isFinite(total) || total < 0) {
    return '0:00';
  }
  const whole = Math.floor(total);
  const seconds = whole % 60;
  const minutes = Math.floor(whole / 60) % 60;
  const hours = Math.floor(whole / 3600);
  const ss = String(seconds).padStart(2, '0');
  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, '0')}:${ss}`;
  }
  return `${minutes}:${ss}`;
}
