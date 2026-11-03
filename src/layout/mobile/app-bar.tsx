import type { ReactNode } from 'react';
import styles from './app-bar.module.css';

type AppBarProps = {
  title: string;
  subtitle?: string;
  children?: ReactNode;
};

/** 移动页顶栏。标题在左，操作在右。 */
export function AppBar({ title, subtitle, children }: AppBarProps) {
  return (
    <header className={styles.bar}>
      <div className={styles.copy}>
        <h1>{title}</h1>
        {subtitle ? <p>{subtitle}</p> : null}
      </div>
      {children ? <div className={styles.actions}>{children}</div> : null}
    </header>
  );
}
