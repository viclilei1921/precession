import { PlusIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import type { Book, BookStatus } from '@/bridge/library';
import styles from '@/components/record.module.css';
import { usePageSearch } from '@/hooks/search';
import { bookListQuery } from '@/query/library/query';
import { PATH } from '@/router/path';
import { errorMessage } from '@/utils/error';
import { matchesQuery } from '@/utils/search';
import { BookForm, bookStatuses } from './components/book-form';

export function LibraryPage() {
  const navigate = useNavigate();
  const search = useSearch({ from: '/library' });
  const booksQuery = useQuery(bookListQuery);
  const [status, setStatus] = useState<BookStatus>('reading');
  const [editing, setEditing] = useState<Book | 'new' | null>(null);
  const query = usePageSearch('搜索书库');
  const books = (booksQuery.data ?? []).filter(
    (book) => book.status === status && matchesQuery(query, book.title, book.author)
  );

  useEffect(() => {
    if (!search.create) {
      return;
    }
    setEditing('new');
    void navigate({ to: PATH.library, search: {}, replace: true });
  }, [navigate, search.create]);

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>书库</h1>
        <p className={styles.hint}>想读、在读和读完。点进一本书，记书摘和笔记。</p>
        <div className={styles.segments} role="tablist" aria-label="书架">
          {bookStatuses.map((item) => (
            <button
              key={item.status}
              type="button"
              className={styles.segment}
              data-on={status === item.status ? 'true' : undefined}
              onClick={() => setStatus(item.status)}
            >
              <item.Icon className={styles.icon} weight="regular" />
              {item.label}
            </button>
          ))}
        </div>
        <button type="button" className={styles.primary} onClick={() => setEditing('new')}>
          <PlusIcon className={styles.icon} weight="regular" />
          添加书
        </button>
      </header>
      {booksQuery.error ? <p className={styles.error}>{errorMessage(booksQuery.error)}</p> : null}
      {editing !== null ? (
        <div className={styles.card}>
          <h2 className={styles.cardTitle}>{editing === 'new' ? '添加书' : '编辑书'}</h2>
          <BookForm
            key={editing === 'new' ? 'new' : editing.id}
            book={editing === 'new' ? null : editing}
            status={status}
            onClose={() => setEditing(null)}
          />
        </div>
      ) : null}
      <div className={styles.card}>
        {booksQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}
        {!booksQuery.isPending && books.length === 0 ? (
          <p className={styles.empty}>{query.trim() ? '没有匹配的书。' : '这个书架还是空的。'}</p>
        ) : null}
        {books.map((book) => (
          <button
            key={book.id}
            type="button"
            className={styles.row}
            onClick={() => void navigate({ to: PATH.bookReader, params: { bookId: book.id } })}
          >
            <span className={styles.rowTitle}>
              {book.title}
              {book.author ? ` · ${book.author}` : ''}
            </span>
            <span className={styles.meta}>
              {book.rating != null ? `${book.rating} 分 · ` : ''}
              {Math.round(book.progress * 100)}%
            </span>
          </button>
        ))}
      </div>
    </section>
  );
}
