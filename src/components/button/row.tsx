import clsx from 'clsx';
import type { MouseEventHandler, ReactNode } from 'react';
import { rippleHost, useRipple } from '@/components/ripple';
import styles from './row.module.css';

type RowVariant = 'list' | 'feed' | 'card' | 'title' | 'trigger';

type RowButtonProps = {
  variant?: RowVariant;
  disabled?: boolean;
  className?: string;
  children: ReactNode;
  onClick?: MouseEventHandler<HTMLButtonElement>;
};

const variantClass: Record<RowVariant, string> = {
  card: styles.card,
  feed: styles.feed,
  list: styles.list,
  title: styles.title,
  trigger: styles.trigger
};

/** 整行、卡片或字段里的文字按钮。 */
export function RowButton({ variant = 'list', disabled, className, children, onClick }: RowButtonProps) {
  const ripple = useRipple();

  return (
    <button
      type="button"
      className={clsx(rippleHost, variantClass[variant], className)}
      disabled={disabled}
      onClick={onClick}
      onPointerDown={ripple.onPointerDown}
      onKeyDown={ripple.onKeyDown}
    >
      {ripple.nodes}
      {children}
    </button>
  );
}
