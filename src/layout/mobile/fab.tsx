import { PlusIcon } from '@phosphor-icons/react';
import type { MouseEventHandler } from 'react';
import styles from './fab.module.css';

type FabProps = {
  label: string;
  onClick: MouseEventHandler<HTMLButtonElement>;
};

/** 右下角主操作。计划和手记各放一个。 */
export function Fab({ label, onClick }: FabProps) {
  return (
    <button type="button" className={styles.fab} onClick={onClick}>
      <PlusIcon weight="bold" />
      {label}
    </button>
  );
}
