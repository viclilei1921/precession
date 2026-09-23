import styles from './page.module.css';

export function TodayPage() {
  return (
    <section className={styles.page}>
      <h1 className={styles.title}>今天</h1>
      <p className={styles.hint}>今天要做的事、随手记的一笔，以及今天新增的记录，会出现在这里。</p>
    </section>
  );
}
