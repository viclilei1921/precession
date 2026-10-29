import clsx from 'clsx';
import type { MouseEventHandler, ReactNode } from 'react';
import { rippleHost, useRipple } from '@/components/ripple';
import styles from './chip.module.css';

/** 胶囊按钮的色调 */
type ChipTone = 'journal' | 'growth' | 'library' | 'plan';

/** 胶囊按钮的属性 */
type ChipButtonProps = {
  /** 胶囊按钮是否选中 */
  selected?: boolean;
  /** 胶囊按钮的色调 */
  tone?: ChipTone;
  /** 胶囊按钮的类名 */
  className?: string;
  /** 胶囊按钮的子元素 */
  children: ReactNode;
  /** 胶囊按钮的点击事件 */
  onClick?: MouseEventHandler<HTMLButtonElement>;
};

/** 胶囊按钮。筛选、入口和选择都用它。带 tone 时用业务浅底，否则选中后反色。
 * @param selected - 胶囊按钮是否选中
 * @param tone - 胶囊按钮的色调
 * @param className - 胶囊按钮的类名
 * @param children - 胶囊按钮的子元素
 * @param onClick - 胶囊按钮的点击事件
 */
export function ChipButton({ selected, tone, className, children, onClick }: ChipButtonProps) {
  const ripple = useRipple();

  return (
    <button
      type="button"
      className={clsx(rippleHost, styles.chip, className)}
      data-on={selected ? 'true' : undefined}
      data-tone={tone}
      onClick={onClick}
      onPointerDown={ripple.onPointerDown}
      onKeyDown={ripple.onKeyDown}
    >
      {ripple.nodes}
      {children}
    </button>
  );
}
