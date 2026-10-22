import { BookOpenIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useCardLayout } from '@/features/today/board/board';
import { bookListQuery } from '@/query/library/query';
import { timelineListQuery } from '@/query/timeline/query';
import { PATH } from '@/router/path';
import { errorMessage } from '@/utils/error';
import styles from './tile.module.css';
import { TileHead } from './tile-head';

export function LibraryCard() {
  const navigate = useNavigate();
  const booksQuery = useQuery(bookListQuery);
  const excerptsQuery = useQuery(timelineListQuery(undefined, undefined, ['excerpt']));
  const reading = [...(booksQuery.data ?? [])]
    .filter((book) => book.status === 'reading')
    .sort((left, right) => right.updatedAt - left.updatedAt)[0];
  const excerpt = excerptsQuery.data?.[0];
  const quote = excerpt ? excerpt.title.trim() || excerpt.body.trim() : '';
  const percent = reading ? Math.round(reading.progress * 100) : 0;
  const error = booksQuery.error ?? excerptsQuery.error;

  useCardLayout('library', `${booksQuery.isPending}:${reading?.id ?? ''}:${percent}:${quote}`);

  return (
    <button type="button" className={styles.card} onClick={() => void navigate({ to: PATH.library })}>
      <TileHead tone="library" icon={BookOpenIcon} title="书库" />
      {error ? <p className={styles.error}>{errorMessage(error)}</p> : null}
      {reading ? (
        <>
          <p className={styles.lead}>在读《{reading.title}》</p>
          <div className={styles.bar} role="img" aria-label={`已读 ${percent}%`}>
            <span data-tone="library" style={{ width: `${percent}%` }} />
          </div>
        </>
      ) : (
        <p className={styles.note}>{booksQuery.isPending ? '正在读取…' : '没有在读的书'}</p>
      )}
      {quote ? <p className={styles.quote}>「{quote}」</p> : null}
    </button>
  );
}
