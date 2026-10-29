import { CaretDoubleRightIcon } from '@phosphor-icons/react';
import clsx from 'clsx';
import type { ReactNode } from 'react';
import { IconButton } from '@/components/button/icon';
import { rippleHost, useRipple } from '@/components/ripple';
import styles from '../drawer.module.css';

type DrawerScrimProps = {
  open: boolean;
  onClose: () => void;
};

/** 很浅的遮罩，用来点空白处关闭抽屉 */
export function DrawerScrim({ open, onClose }: DrawerScrimProps) {
  return (
    <button
      type="button"
      className={styles.scrim}
      data-open={open ? 'true' : undefined}
      tabIndex={-1}
      aria-label="关闭"
      onClick={onClose}
    />
  );
}

type DrawerPanelProps = {
  open: boolean;
  /** 计划表单用宽面板。清单保持窄面板 */
  wide?: boolean;
  title: string;
  label: string;
  children: ReactNode;
  onClose: () => void;
};

/** 从右侧滑出的抽屉 */
export function DrawerPanel({ open, wide, title, label, children, onClose }: DrawerPanelProps) {
  return (
    <aside
      className={styles.panel}
      data-open={open ? 'true' : undefined}
      data-wide={wide ? 'true' : undefined}
      aria-label={label}
    >
      <div className={styles.head}>
        <h2>{title}</h2>
        <IconButton label={`收起${title}`} onClick={onClose}>
          <CaretDoubleRightIcon weight="regular" />
        </IconButton>
      </div>
      {children}
    </aside>
  );
}

/** 抽屉里的添加按钮。按下时有水波纹 */
export function DrawerAdd({ children, onClick }: { children: ReactNode; onClick: () => void }) {
  const ripple = useRipple();

  return (
    <button
      type="button"
      className={clsx(rippleHost, styles.add)}
      onClick={onClick}
      onPointerDown={ripple.onPointerDown}
      onKeyDown={ripple.onKeyDown}
    >
      {ripple.nodes}
      {children}
    </button>
  );
}
