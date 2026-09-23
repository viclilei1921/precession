import { useParams } from '@tanstack/react-router';
import styles from './page.module.css';

export function ReaderPage() {
  const { bookId } = useParams({ from: '/library/$bookId' });

  return (
    <section className={styles.page}>
      <h1 className={styles.title}>阅读</h1>
      <p className={styles.hint}>划线和批注会留在这本书上。</p>
      <p className={styles.hint}>{bookId}</p>
    </section>
  );
}
