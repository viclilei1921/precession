import clsx from 'clsx';
import type { MouseEventHandler, ReactNode } from 'react';
import { rippleHost, useRipple } from '@/components/ripple';
import styles from './segment.module.css';

type SegmentButtonProps = {
  selected?: boolean;
  className?: string;
  children: ReactNode;
  onClick?: MouseEventHandler<HTMLButtonElement>;
};

/** 页头分段里的一项。选中用 data-on。 */
export function SegmentButton({ selected, className, children, onClick }: SegmentButtonProps) {
  const ripple = useRipple();

  return (
    <button
      type="button"
      className={clsx(rippleHost, styles.segment, className)}
      data-on={selected ? 'true' : undefined}
      onClick={onClick}
      onPointerDown={ripple.onPointerDown}
      onKeyDown={ripple.onKeyDown}
    >
      {ripple.nodes}
      {children}
    </button>
  );
}
