import styles from './index.module.css';

type ProgressTone = 'plan' | 'library';

type ProgressProps = {
  /** 0 到 100 */
  value: number;
  label?: string;
  tone?: ProgressTone;
};

/** 进度条。 */
export function Progress({ value, label = '进度', tone = 'plan' }: ProgressProps) {
  const next = Math.min(100, Math.max(0, value));

  return (
    <div
      className={styles.track}
      role="progressbar"
      aria-label={label}
      aria-valuenow={Math.round(next)}
      aria-valuemin={0}
      aria-valuemax={100}
      data-tone={tone}
    >
      <div className={styles.fill} style={{ width: `${next}%` }} />
    </div>
  );
}
