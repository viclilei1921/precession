import type { Icon } from '@phosphor-icons/react';
import { BookmarkSimpleIcon, BookOpenIcon, CheckCircleIcon, PlusIcon } from '@phosphor-icons/react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import type { Book, BookStatus } from '@/bridge/library';
import { bookCreate, bookDelete, bookUpdate } from '@/bridge/library';
import { Dialog } from '@/components/dialog';
import styles from '@/components/record.module.css';
import { usePageSearch } from '@/layout/titleBar/searchHook';
import { bookListQuery, refreshLibrary } from '@/query/library/query';
import { PATH } from '@/router/path';
import { fromDateInputValue, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import { matchesQuery } from '@/utils/search';

const statuses: { status: BookStatus; label: string; Icon: Icon }[] = [
  { status: 'want', label: '想读', Icon: BookmarkSimpleIcon },
  { status: 'reading', label: '在读', Icon: BookOpenIcon },
  { status: 'finished', label: '读完', Icon: CheckCircleIcon }
];

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
          {statuses.map((item) => (
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
      {editing !== null ? (
        <BookDialog
          key={editing === 'new' ? 'new' : editing.id}
          book={editing === 'new' ? null : editing}
          status={status}
          onClose={() => setEditing(null)}
        />
      ) : null}
    </section>
  );
}

export function BookDialog({ book, status, onClose }: { book: Book | null; status: BookStatus; onClose: () => void }) {
  const queryClient = useQueryClient();
  const [title, setTitle] = useState(book?.title ?? '');
  const [author, setAuthor] = useState(book?.author ?? '');
  const [draftStatus, setDraftStatus] = useState<BookStatus>(book?.status ?? status);
  const [progress, setProgress] = useState(String(Math.round((book?.progress ?? 0) * 100)));
  const [rating, setRating] = useState(book?.rating == null ? '' : String(book.rating));
  const [startedAt, setStartedAt] = useState(toDateInputValue(book?.startedAt ?? null));
  const [finishedAt, setFinishedAt] = useState(toDateInputValue(book?.finishedAt ?? null));
  const [confirmDelete, setConfirmDelete] = useState(false);
  const save = useMutation({
    mutationFn: () => {
      const ratio = Math.min(100, Math.max(0, Number(progress) || 0)) / 100;
      const input = {
        title,
        author,
        coverPath: book?.coverPath ?? '',
        status: draftStatus,
        progress: ratio,
        rating: rating === '' ? null : Number(rating),
        startedAt: fromDateInputValue(startedAt),
        finishedAt: fromDateInputValue(finishedAt)
      };
      return book ? bookUpdate(book.id, input) : bookCreate(input);
    },
    onSuccess: async () => {
      await refreshLibrary(queryClient);
      onClose();
    }
  });
  const remove = useMutation({
    mutationFn: () => (book ? bookDelete(book.id) : Promise.reject(new Error('记录不存在'))),
    onSuccess: async () => {
      await refreshLibrary(queryClient);
      onClose();
    }
  });
  const pending = save.isPending || remove.isPending;

  return (
    <Dialog title={book ? '编辑书' : '添加书'} onClose={onClose}>
      <form
        className={styles.stack}
        onSubmit={(event) => {
          event.preventDefault();
          save.mutate();
        }}
      >
        <label className={styles.field}>
          书名
          <input value={title} required onChange={(event) => setTitle(event.target.value)} />
        </label>
        <label className={styles.field}>
          作者
          <input value={author} onChange={(event) => setAuthor(event.target.value)} />
        </label>
        <div className={styles.pair}>
          <label className={styles.field}>
            状态
            <select value={draftStatus} onChange={(event) => setDraftStatus(event.target.value as BookStatus)}>
              {statuses.map((item) => (
                <option key={item.status} value={item.status}>
                  {item.label}
                </option>
              ))}
            </select>
          </label>
          <label className={styles.field}>
            进度（%）
            <input
              type="number"
              min={0}
              max={100}
              value={progress}
              onChange={(event) => setProgress(event.target.value)}
            />
          </label>
        </div>
        <div className={styles.pair}>
          <label className={styles.field}>
            评分
            <select value={rating} onChange={(event) => setRating(event.target.value)}>
              <option value="">未评分</option>
              {[1, 2, 3, 4, 5].map((score) => (
                <option key={score} value={score}>
                  {score}
                </option>
              ))}
            </select>
          </label>
          <label className={styles.field}>
            开始
            <input type="date" value={startedAt} onChange={(event) => setStartedAt(event.target.value)} />
          </label>
        </div>
        <label className={styles.field}>
          读完
          <input type="date" value={finishedAt} onChange={(event) => setFinishedAt(event.target.value)} />
        </label>
        {save.error ? <p className={styles.error}>{errorMessage(save.error)}</p> : null}
        {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
        <div className={styles.actions}>
          {book ? (
            <button
              type="button"
              className={styles.danger}
              disabled={pending}
              onClick={() => {
                if (!confirmDelete) {
                  setConfirmDelete(true);
                  return;
                }
                remove.mutate();
              }}
            >
              {confirmDelete ? '确认删除' : '删除'}
            </button>
          ) : null}
          <span className={styles.spacer} />
          <button type="button" className={styles.ghost} disabled={pending} onClick={onClose}>
            取消
          </button>
          <button type="submit" className={styles.primary} disabled={pending}>
            保存
          </button>
        </div>
      </form>
    </Dialog>
  );
}
