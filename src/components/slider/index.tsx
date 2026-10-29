import clsx from 'clsx';
import type { KeyboardEvent, PointerEvent } from 'react';
import { useId, useState } from 'react';
import styles from './index.module.css';

type SliderProps = {
  /** 标题，同时作为读屏名称 */
  label: string;
  min: number;
  max: number;
  /** 拖动和方向键的步进。默认 1 */
  step?: number;
  /** Page Up / Page Down 的步进。默认是 `step` 的 10 倍 */
  shiftStep?: number;
  value: number;
  className?: string;
  onChange: (value: number) => void;
};

function decimalsOf(step: number) {
  const text = step.toString();
  const dot = text.indexOf('.');
  return dot === -1 ? 0 : text.length - dot - 1;
}

function clampStep(raw: number, min: number, max: number, step: number) {
  const steps = Math.round((raw - min) / step);
  const next = min + steps * step;
  const rounded = Number(next.toFixed(decimalsOf(step)));
  return Math.min(max, Math.max(min, rounded));
}

/**
 * Material 风格滑块。
 *
 * 轨道左侧是当前值，圆点跟随指针。悬停、聚焦或拖动时，圆点上方显示数值。
 */
export function Slider({ label, min, max, step = 1, shiftStep, value, className, onChange }: SliderProps) {
  const labelId = useId();
  const pageStep = shiftStep ?? step * 10;
  const [dragging, setDragging] = useState(false);
  const span = max - min;
  const percent = span <= 0 ? 0 : ((value - min) / span) * 100;

  function valueAt(element: HTMLElement, clientX: number) {
    const rect = element.getBoundingClientRect();
    if (rect.width <= 0) {
      return value;
    }
    const ratio = (clientX - rect.left) / rect.width;
    return clampStep(min + ratio * span, min, max, step);
  }

  function nudge(next: number) {
    onChange(clampStep(next, min, max, step));
  }

  function onPointerDown(event: PointerEvent<HTMLDivElement>) {
    if (event.button !== 0) {
      return;
    }
    event.currentTarget.setPointerCapture(event.pointerId);
    event.currentTarget.focus();
    setDragging(true);
    onChange(valueAt(event.currentTarget, event.clientX));
  }

  function onPointerMove(event: PointerEvent<HTMLDivElement>) {
    if (!event.currentTarget.hasPointerCapture(event.pointerId)) {
      return;
    }
    onChange(valueAt(event.currentTarget, event.clientX));
  }

  function onPointerUp(event: PointerEvent<HTMLDivElement>) {
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    setDragging(false);
  }

  function onKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    if (event.key === 'ArrowRight' || event.key === 'ArrowUp') {
      event.preventDefault();
      nudge(value + step);
      return;
    }
    if (event.key === 'ArrowLeft' || event.key === 'ArrowDown') {
      event.preventDefault();
      nudge(value - step);
      return;
    }
    if (event.key === 'PageUp') {
      event.preventDefault();
      nudge(value + pageStep);
      return;
    }
    if (event.key === 'PageDown') {
      event.preventDefault();
      nudge(value - pageStep);
      return;
    }
    if (event.key === 'Home') {
      event.preventDefault();
      onChange(min);
      return;
    }
    if (event.key === 'End') {
      event.preventDefault();
      onChange(max);
    }
  }

  return (
    <div className={clsx(styles.root, className)}>
      <div className={styles.caption}>
        <span id={labelId}>{label}</span>
        <span className={styles.captionValue}>{value}</span>
      </div>
      <div
        className={styles.slider}
        role="slider"
        tabIndex={0}
        aria-labelledby={labelId}
        aria-orientation="horizontal"
        aria-valuemin={min}
        aria-valuemax={max}
        aria-valuenow={value}
        aria-valuetext={String(value)}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
        onKeyDown={onKeyDown}
      >
        <span className={styles.rail} />
        <span className={styles.track} style={{ width: `${percent}%` }} />
        <span className={styles.thumb} style={{ left: `${percent}%` }} data-on={dragging ? 'true' : undefined}>
          <span className={styles.value}>{value}</span>
        </span>
      </div>
    </div>
  );
}
