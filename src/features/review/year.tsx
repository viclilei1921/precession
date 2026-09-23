import styles from './page.module.css';

export function YearPage() {
  return (
    <section className={styles.page}>
      <h1 className={styles.title}>年度之书</h1>
      <p className={styles.hint}>这一年的计划、手记、成长和书单会汇总在这里。</p>
    </section>
  );
}
