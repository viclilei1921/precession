import styles from './page.module.css';

export function LibraryPage() {
  return (
    <section className={styles.page}>
      <h1 className={styles.title}>书库</h1>
      <p className={styles.hint}>想读、在读和读过的书会放在这里。</p>
    </section>
  );
}
