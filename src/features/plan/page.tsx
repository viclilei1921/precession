import styles from './page.module.css';

export function PlanPage() {
  return (
    <section className={styles.page}>
      <h1 className={styles.title}>计划</h1>
      <p className={styles.hint}>待办、收集箱和日历是同一份计划的三种看法。</p>
    </section>
  );
}
