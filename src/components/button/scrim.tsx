import type { MouseEventHandler } from 'react';
import { useTranslation } from 'react-i18next';
import styles from './scrim.module.css';

type ScrimButtonProps = {
  label?: string;
  onClick?: MouseEventHandler<HTMLButtonElement>;
};

/** 盖住对话框背后的透明按钮。 */
export function ScrimButton({ label, onClick }: ScrimButtonProps) {
  const { t } = useTranslation('ui');
  const name = label ?? t('common:button.close');
  return <button type="button" className={styles.scrim} aria-label={name} onClick={onClick} />;
}
