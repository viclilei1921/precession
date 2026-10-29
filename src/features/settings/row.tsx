import type { ReactNode } from 'react';
import styles from './row.module.css';

type SettingRowProps = {
  title: string;
  hint: ReactNode;
  /** 说明被截断时，悬停可以看到全文 */
  hintTitle?: string;
  children?: ReactNode;
};

/** 设置卡片里的一行：标题、说明，右侧是控件。 */
export function SettingRow({ title, hint, hintTitle, children }: SettingRowProps) {
  return (
    <div className={styles.row}>
      <div className={styles.copy}>
        <div className={styles.title}>{title}</div>
        <p className={styles.hint} title={hintTitle}>
          {hint}
        </p>
      </div>
      {children ? <div className={styles.control}>{children}</div> : null}
    </div>
  );
}
