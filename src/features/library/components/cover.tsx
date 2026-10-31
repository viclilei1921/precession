import type { Book } from '@/bridge/library';
import { bookFormat, displayTitle } from '../book';
import styles from './cover.module.css';

const SPINES = [
  { paper: '#e8eef8', ink: '#1e3a5f', spine: '#3c6eae' },
  { paper: '#e7f3ef', ink: '#1d3d34', spine: '#2f7d6d' },
  { paper: '#f6efe2', ink: '#4a3518', spine: '#9a6b2f' },
  { paper: '#f6ebe4', ink: '#4a2a1c', spine: '#a85a32' },
  { paper: '#eeeaf6', ink: '#34284d', spine: '#6a4ea3' },
  { paper: '#e8eef2', ink: '#243846', spine: '#3e5c73' }
];

function tone(id: string) {
  let hash = 0;
  for (const char of id) {
    hash = (hash + char.charCodeAt(0)) % SPINES.length;
  }
  return SPINES[hash] ?? SPINES[0];
}

/** 字脊封面。不使用文件里的封面图 */
export function Cover({ book, compact }: { book: Book; compact?: boolean }) {
  const color = tone(book.id);
  const format = bookFormat(book);
  const reading = book.status === 'reading';

  return (
    <span
      className={compact ? styles.compact : styles.cover}
      style={{ background: color.paper, color: color.ink, ['--spine' as string]: color.spine }}
    >
      <span className={styles.name}>{displayTitle(book.title)}</span>
      {format ? <span className={styles.format}>{format}</span> : null}
      {reading ? (
        <span className={styles.progress} role="img" aria-label={`已读 ${Math.round(book.progress * 100)}%`}>
          <span style={{ width: `${Math.round(book.progress * 100)}%` }} />
        </span>
      ) : null}
    </span>
  );
}
