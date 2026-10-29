import clsx from 'clsx';
import { useState } from 'react';
import { clock } from './preview';
import styles from './scrub.module.css';

type ScrubProps = {
  current: number;
  duration: number;
  buffered?: number;
  /** 悬停时在滑块上方显示时间 */
  tip?: boolean;
  /** 深色预览层上用浅色轨道 */
  invert?: boolean;
  label?: string;
  className?: string;
  onSeek: (time: number) => void;
};

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

/** 视频进度。左侧是已播放，中间淡一点的是已缓冲。 */
export function Scrub({
  current,
  duration,
  buffered = 0,
  tip = false,
  invert = false,
  label = '进度',
  className,
  onSeek
}: ScrubProps) {
  const [hover, setHover] = useState<{ ratio: number; time: number } | null>(null);
  const played = duration > 0 ? Math.min(100, (current / duration) * 100) : 0;
  const loaded = duration > 0 ? Math.min(100, (buffered / duration) * 100) : 0;

  function show(ratio: number) {
    if (!tip || duration <= 0) {
      return;
    }
    const next = clamp(ratio, 0, 1);
    setHover({ ratio: next, time: next * duration });
  }

  return (
    <span
      className={clsx(styles.root, className)}
      data-tone={invert ? 'invert' : undefined}
      onPointerMove={(event) => {
        const rect = event.currentTarget.getBoundingClientRect();
        if (rect.width <= 0) {
          return;
        }
        show((event.clientX - rect.left) / rect.width);
      }}
      onPointerLeave={() => setHover(null)}
    >
      {hover ? (
        <span className={styles.tip} style={{ left: `${hover.ratio * 100}%` }}>
          {clock(hover.time)}
        </span>
      ) : null}
      <span className={styles.buffered} style={{ width: `${loaded}%` }} />
      <span className={styles.played} style={{ width: `${played}%` }} />
      <input
        className={styles.input}
        type="range"
        min={0}
        max={duration || 0}
        step={0.01}
        value={Number.isFinite(current) ? current : 0}
        aria-label={label}
        disabled={!duration}
        onChange={(event) => {
          const time = Number(event.target.value);
          onSeek(time);
          if (duration > 0) {
            show(time / duration);
          }
        }}
      />
    </span>
  );
}
