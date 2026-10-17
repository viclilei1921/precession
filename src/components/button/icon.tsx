import clsx from 'clsx';
import type { MouseEventHandler, ReactNode } from 'react';
import { rippleHost, useRipple } from '@/components/ripple';
import styles from './icon.module.css';

/** 图标按钮的种类 */
type IconButtonVariant = 'standard' | 'filled' | 'outlined';
/** 图标按钮的大小 */
type IconButtonSize = 'default' | 'compact' | 'plain';
/** 图标按钮的色调 */
type IconButtonTone = 'destructive';

/** 图标按钮的属性 */
type IconButtonProps = {
  /** 无可见文字，同时用作 aria-label 和 title */
  label: string;
  /** 图标按钮的种类 */
  variant?: IconButtonVariant;
  /** 标题栏用 compact。plain 不占固定方框，给完成勾选用。 */
  size?: IconButtonSize;
  /** 图标按钮的色调 */
  tone?: IconButtonTone;
  /** 选中时铺一层浅底，例如侧栏已收起 */
  selected?: boolean;
  /** 传入时写入 aria-expanded */
  expanded?: boolean;
  /** 图标按钮的类型 */
  type?: 'button' | 'submit' | 'reset';
  /** 图标按钮是否禁用 */
  disabled?: boolean;
  /** 图标按钮的类名 */
  className?: string;
  /** 图标按钮的子元素 */
  children: ReactNode;
  /** 图标按钮的点击事件 */
  onClick?: MouseEventHandler<HTMLButtonElement>;
};

/** 图标按钮的种类对应的类名 */
const variantClass: Record<IconButtonVariant, string> = {
  filled: styles.filled,
  outlined: styles.outlined,
  standard: styles.standard
};

/** 圆形图标按钮。图标作为 children 放入。 */
export function IconButton({
  label,
  variant = 'standard',
  size = 'default',
  tone,
  selected,
  expanded,
  type = 'button',
  disabled,
  className,
  children,
  onClick
}: IconButtonProps) {
  const ripple = useRipple();

  return (
    <button
      type={type}
      className={clsx(
        rippleHost,
        styles.button,
        variantClass[variant],
        size === 'compact' && styles.compact,
        size === 'plain' && styles.plain,
        className
      )}
      data-tone={tone}
      data-on={selected ? 'true' : undefined}
      aria-label={label}
      aria-expanded={expanded}
      title={label}
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
