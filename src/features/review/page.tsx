import styles from './page.module.css';

export function ReviewPage() {
  return (
    <section className={styles.page}>
      <h1 className={styles.title}>回顾</h1>
      <p className={styles.hint}>四块记录会按时间重新读出来。</p>
    </section>
  );
}
