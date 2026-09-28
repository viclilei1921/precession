import styles from './page.module.css';

type EmptyStateProps = {
  title: string;
  hint: string;
};

export function EmptyState({ title, hint }: EmptyStateProps) {
  return (
    <section className={styles.page}>
      <h1 className={styles.title}>{title}</h1>
      <p className={styles.hint}>{hint}</p>
    </section>
  );
}
