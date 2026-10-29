import clsx from 'clsx';
import type { ReactNode } from 'react';
import styles from './index.module.css';

type CardProps = {
  /** 卡片标题 */
  title?: string;
  /** 标题右侧的附加内容 */
  extra?: ReactNode;
  /** 锚点，给页内跳转用 */
  id?: string;
  className?: string;
  children: ReactNode;
};

/** 白底描边的表面。不读取数据。 */
export function Card({ title, extra, id, className, children }: CardProps) {
  return (
    <article id={id} className={clsx(styles.card, className)}>
      {title || extra ? (
        <header className={styles.head}>
          {title ? <h2 className={styles.title}>{title}</h2> : null}
          {extra ? <div className={styles.extra}>{extra}</div> : null}
        </header>
      ) : null}
      {children}
    </article>
  );
}
