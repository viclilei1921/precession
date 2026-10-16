import type { Icon } from '@phosphor-icons/react';
import { BookmarkSimpleIcon, BookOpenIcon, CheckCircleIcon } from '@phosphor-icons/react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import type { Book, BookStatus } from '@/bridge/library';
import { bookCreate, bookDelete, bookUpdate } from '@/bridge/library';
import { DateField } from '@/components/date-field';
import styles from '@/components/record.module.css';
import { refreshLibrary } from '@/query/library/query';
import { fromDateInputValue, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';

export const bookStatuses: { status: BookStatus; label: string; Icon: Icon }[] = [
  { status: 'want', label: '想读', Icon: BookmarkSimpleIcon },
  { status: 'reading', label: '在读', Icon: BookOpenIcon },
  { status: 'finished', label: '读完', Icon: CheckCircleIcon }
];

export function BookForm({ book, status, onClose }: { book: Book | null; status: BookStatus; onClose: () => void }) {
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
            {bookStatuses.map((item) => (
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
        <DateField label="开始" value={startedAt} onChange={setStartedAt} />
      </div>
      <DateField label="读完" value={finishedAt} onChange={setFinishedAt} />
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
  );
}
