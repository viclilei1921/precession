import clsx from 'clsx';
import type { MouseEventHandler, ReactNode } from 'react';
import { rippleHost, useRipple } from '@/components/ripple';
import styles from './day.module.css';

type DayVariant = 'date' | 'month';

type DayButtonProps = {
  variant?: DayVariant;
  selected?: boolean;
  today?: boolean;
  outside?: boolean;
  off?: boolean;
  className?: string;
  children: ReactNode;
  onClick?: MouseEventHandler<HTMLButtonElement>;
};

/** 日历中的一天。date 用于日期字段，month 用于计划月历。 */
export function DayButton({
  variant = 'date',
  selected,
  today,
  outside,
  off,
  className,
  children,
  onClick
}: DayButtonProps) {
  const ripple = useRipple();

  return (
    <button
      type="button"
      className={clsx(rippleHost, variant === 'month' ? styles.month : styles.date, className)}
      data-selected={selected ? 'true' : undefined}
      data-today={today ? 'true' : undefined}
      data-outside={outside ? 'true' : undefined}
      data-off={off ? 'true' : undefined}
      onClick={onClick}
      onPointerDown={ripple.onPointerDown}
      onKeyDown={ripple.onKeyDown}
    >
      {ripple.nodes}
      {children}
    </button>
  );
}
