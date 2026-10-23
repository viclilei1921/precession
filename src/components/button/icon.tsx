import clsx from 'clsx';
import type { MouseEventHandler, ReactNode } from 'react';
import { rippleHost, useRipple } from '@/components/ripple';
import styles from './icon.module.css';

/** 图标按钮的种类 */
type IconButtonVariant = 'standard' | 'filled' | 'outlined';
/** 图标按钮的大小 */
type IconButtonSize = 'default' | 'compact' | 'plain';
/** 图标按钮的外形。圆形是默认，圆角和直角给顶栏、组合按钮用 */
type IconButtonShape = 'circle' | 'rounded' | 'square';
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
  /** 圆形、圆角方块，或直角。组合里用直角，外框负责圆角 */
  shape?: IconButtonShape;
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

/** 图标按钮。图标作为 children 放入。默认圆形，也可以是圆角或直角。 */
export function IconButton({
  label,
  variant = 'standard',
  size = 'default',
  shape = 'circle',
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
        shape === 'rounded' && styles.rounded,
        shape === 'square' && styles.square,
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
