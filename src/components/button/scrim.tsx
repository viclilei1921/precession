import type { MouseEventHandler } from 'react';
import styles from './scrim.module.css';

type ScrimButtonProps = {
  label?: string;
  onClick?: MouseEventHandler<HTMLButtonElement>;
};

/** 盖住对话框背后的透明按钮。 */
export function ScrimButton({ label = '关闭', onClick }: ScrimButtonProps) {
  return <button type="button" className={styles.scrim} aria-label={label} onClick={onClick} />;
}
