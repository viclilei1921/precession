import { useParams } from '@tanstack/react-router';
import styles from './page.module.css';

export function JournalEditorPage() {
  const { entryId } = useParams({ from: '/journal/$entryId' });

  return (
    <section className={styles.page}>
      <h1 className={styles.title}>编辑手记</h1>
      <p className={styles.hint}>正文、图片视频和书摘会写在这篇里。</p>
      <p className={styles.hint}>{entryId}</p>
    </section>
  );
}
