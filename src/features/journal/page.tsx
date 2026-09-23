import styles from './page.module.css';

export function JournalPage() {
  return (
    <section className={styles.page}>
      <h1 className={styles.title}>手记</h1>
      <p className={styles.hint}>日记、灵感和写作会列在这里。</p>
    </section>
  );
}
