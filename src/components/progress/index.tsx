import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import styles from './index.module.css';

/** 进度条的色调 */
type ProgressTone = 'plan' | 'library' | 'toolbox';

/** 进度条的属性 */
type ProgressProps = {
  /** 0 到 100 */
  value: number;
  /** 读屏名称 */
  label?: string;
  /** 进度条的色调 */
  tone?: ProgressTone;
  /** 进度条的类名 */
  className?: string;
};

/**
 * Material 风格进度条。
 *
 * 轨道用色调浅底，填充从左向右按数值长出。数值变化时按线性过渡，避免缓动让进度看起来超前或滞后。
 */
export function Progress({ value, label, tone = 'plan', className }: ProgressProps) {
  const { t } = useTranslation('ui');
  const next = Math.min(100, Math.max(0, value));
  const name = label ?? t('progress.label');

  return (
    <div
      className={clsx(styles.track, className)}
      role="progressbar"
      aria-label={name}
      aria-valuenow={Math.round(next)}
      aria-valuemin={0}
      aria-valuemax={100}
      data-tone={tone}
    >
      <div className={styles.fill} style={{ width: `${next}%` }} />
    </div>
  );
}
