import { PlusIcon } from '@phosphor-icons/react';
import clsx from 'clsx';
import type { MouseEventHandler } from 'react';
import { rippleHost, useRipple } from '@/components/ripple';
import styles from './fab.module.css';

type FabProps = {
  label: string;
  onClick: MouseEventHandler<HTMLButtonElement>;
};

/** 右下角主操作。计划和手记各放一个。按下时有水波纹。 */
export function Fab({ label, onClick }: FabProps) {
  const ripple = useRipple();

  return (
    <button
      type="button"
      className={clsx(rippleHost, styles.fab)}
      onClick={onClick}
      onPointerDown={ripple.onPointerDown}
      onKeyDown={ripple.onKeyDown}
    >
      {ripple.nodes}
      <PlusIcon weight="bold" />
      {label}
    </button>
  );
}
