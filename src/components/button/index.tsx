import clsx from 'clsx';
import type { MouseEventHandler, ReactNode } from 'react';
import { rippleHost, useRipple } from '@/components/ripple';
import styles from './index.module.css';

/** 按钮的种类 */
type ButtonVariant = 'filled' | 'outlined' | 'text';
/** 按钮的色调 */
type ButtonTone = 'destructive';

/** 按钮的属性 */
type ButtonProps = {
  /** 按钮的种类 */
  variant?: ButtonVariant;
  /** 按钮的色调 */
  tone?: ButtonTone;
  /** 按钮的类型 */
  type?: 'button' | 'submit' | 'reset';
  /** 按钮是否禁用 */
  disabled?: boolean;
  /** 按钮的类名 */
  className?: string;
  /** 按钮的子元素 */
  children: ReactNode;
  /** 按钮的点击事件 */
  onClick?: MouseEventHandler<HTMLButtonElement>;
};

/** 按钮的种类对应的类名 */
const variantClass: Record<ButtonVariant, string> = {
  filled: styles.filled,
  outlined: styles.outlined,
  text: styles.text
};

/**
 * Material 风格按钮
 *
 * @remarks
 * 图标作为 children 放入组件内部
 *
 * @param variant - 按钮种类
 * @param tone - 按钮色调
 * @param type - 按钮原生类型
 * @param disabled - 是否禁用
 * @param className - 自定义类名
 * @param children - 子元素，一般放图标
 * @param onClick - 点击事件
 */
export function Button({
  variant = 'filled',
  tone,
  type = 'button',
  disabled,
  className,
  children,
  onClick
}: ButtonProps) {
  const ripple = useRipple();

  return (
    <button
      type={type}
      className={clsx(rippleHost, styles.button, variantClass[variant], className)}
      data-tone={tone}
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
