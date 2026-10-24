import { CaretLeftIcon } from '@phosphor-icons/react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate, useParams, useSearch } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import type { Book, BookNote, BookStatus } from '@/bridge/library';
import { bookDelete, bookUpdate } from '@/bridge/library';
import { onTaskUpdate } from '@/bridge/task';
import { Button } from '@/components/button';
import { SegmentButton } from '@/components/button/segment';
import { Scroll } from '@/components/scroll';
import { DrawerPanel, DrawerScrim } from '@/features/plan/components/drawer-frame';
import { bookNoteListQuery, bookQuery, refreshLibrary } from '@/query/library/query';
import { PATH } from '@/router/path';
import { formatMonthDay } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import { bookEncrypted, bookFile, bookStatusLabel, toInput } from './book';
import styles from './book.module.css';
import { Cover } from './components/cover';
import { NoteForm, noteBody } from './components/note-form';

const STATUSES: BookStatus[] = ['reading', 'want', 'finished'];

export function ReaderPage() {
  const { bookId } = useParams({ from: '/library/$bookId' });
  const search = useSearch({ from: '/library/$bookId' });
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const bookQueryResult = useQuery(bookQuery(bookId));
  const notesQuery = useQuery(bookNoteListQuery(bookId));
  const [note, setNote] = useState<BookNote | null | 'new'>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const book = bookQueryResult.data;
  const file = book ? bookFile(book) : undefined;
  const importing = Boolean(search.task) && !file;
  const notes = [...(notesQuery.data ?? [])].sort((left, right) => right.occurredAt - left.occurredAt);
  const update = useMutation({
    mutationFn: (next: Book) => bookUpdate(next.id, toInput(next)),
    onSuccess: async () => {
      await refreshLibrary(queryClient);
    }
  });
  const remove = useMutation({
    mutationFn: () => bookDelete(bookId),
    onSuccess: async () => {
      await refreshLibrary(queryClient);
      void navigate({ to: PATH.library });
    }
  });

  useEffect(() => {
    if (!search.task) {
      return;
    }
    let stop: (() => void) | undefined;
    let cancelled = false;
    void onTaskUpdate((task) => {
      if (task.id !== search.task) {
        return;
      }
      if (task.status === 'completed' || task.status === 'failed' || task.status === 'canceled') {
        void refreshLibrary(queryClient).then(() => {
          void navigate({ to: PATH.bookReader, params: { bookId }, search: {}, replace: true });
        });
      }
    }).then((unlisten) => {
      if (cancelled) {
        unlisten();
        return;
      }
      stop = unlisten;
    });
    return () => {
      cancelled = true;
      stop?.();
    };
  }, [bookId, navigate, queryClient, search.task]);

  useEffect(() => {
    if (!search.task || !file) {
      return;
    }
    void navigate({ to: PATH.bookReader, params: { bookId }, search: {}, replace: true });
  }, [bookId, file, navigate, search.task]);

  function patch(partial: Partial<Book>) {
    if (!book) {
      return;
    }
    update.mutate({ ...book, ...partial });
  }

  return (
    <section className={styles.page}>
      <Button variant="outlined" onClick={() => void navigate({ to: PATH.library })}>
        <CaretLeftIcon weight="regular" />
        返回书架
      </Button>
      <Scroll className={styles.scroll}>
        {bookQueryResult.error ? <p className={styles.note}>{errorMessage(bookQueryResult.error)}</p> : null}
        {book ? (
          <div className={styles.layout}>
            <div className={styles.dossier}>
              <Cover book={book} />
              <h1>{book.title}</h1>
              <p className={styles.meta}>{book.author || '佚名'}</p>
              <div className={styles.segments}>
                {STATUSES.map((status) => (
                  <SegmentButton key={status} selected={book.status === status} onClick={() => patch({ status })}>
                    {bookStatusLabel[status]}
                  </SegmentButton>
                ))}
              </div>
              <div className={styles.stars}>
                {[1, 2, 3, 4, 5].map((star) => (
                  <button
                    key={star}
                    type="button"
                    data-on={(book.rating ?? 0) >= star ? 'true' : undefined}
                    aria-label={`${star} 星`}
                    onClick={() => patch({ rating: book.rating === star ? null : star })}
                  >
                    ★
                  </button>
                ))}
              </div>
              <label className={styles.range}>
                进度 {Math.round(book.progress * 100)}%
                <input
                  type="range"
                  min={0}
                  max={100}
                  value={Math.round(book.progress * 100)}
                  aria-label="阅读进度"
                  onChange={(event) => patch({ progress: Number(event.target.value) / 100 })}
                />
              </label>
              {importing ? <p className={styles.note}>正在导入</p> : null}
              {file || importing ? null : (
                <p className={styles.note}>{bookEncrypted(book) ? '这本书已加密，暂时不能阅读' : '还没有正文'}</p>
              )}
              <div className={styles.actions}>
                {file ? (
                  <Button
                    className={styles.action}
                    onClick={() => void navigate({ to: PATH.bookRead, params: { bookId } })}
                  >
                    继续阅读
                  </Button>
                ) : null}
                <Button
                  className={styles.action}
                  variant="outlined"
                  tone="destructive"
                  disabled={remove.isPending}
                  onClick={() => {
                    if (!confirmDelete) {
                      setConfirmDelete(true);
                      return;
                    }
                    remove.mutate();
                  }}
                >
                  {confirmDelete ? '确认删除' : '删除这本书'}
                </Button>
              </div>
              {update.error ? <p className={styles.note}>{errorMessage(update.error)}</p> : null}
              {remove.error ? <p className={styles.note}>{errorMessage(remove.error)}</p> : null}
            </div>
            <div className={styles.notes}>
              <div className={styles.notesHead}>
                <h2>书摘和笔记</h2>
                <Button onClick={() => setNote('new')}>记一笔</Button>
              </div>
              <div className={styles.list}>
                {notes.map((item) => (
                  <button
                    key={item.id}
                    type="button"
                    className={styles.noteCard}
                    data-highlight={item.highlight ? 'true' : undefined}
                    onClick={() => setNote(item)}
                  >
                    <span className={styles.noteTop}>
                      <span>{item.kind === 'excerpt' ? '书摘' : '笔记'}</span>
                      <span>{formatMonthDay(item.occurredAt)}</span>
                    </span>
                    <span className={styles.body}>{noteBody(item)}</span>
                    <span className={styles.noteMeta}>
                      {item.chapter ? <span>{item.chapter}</span> : null}
                      {item.location ? <span>{item.location}</span> : null}
                    </span>
                  </button>
                ))}
                {!notesQuery.isPending && notes.length === 0 ? (
                  <div className={styles.emptyNotes}>
                    <p>还没有留下句子</p>
                    <span>读到想记住的地方，记一笔就好。</span>
                  </div>
                ) : null}
              </div>
            </div>
          </div>
        ) : null}
      </Scroll>
      <DrawerScrim open={note !== null} onClose={() => setNote(null)} />
      <DrawerPanel
        open={note !== null}
        title={note && note !== 'new' ? '修改' : '记一笔'}
        label="书摘或笔记"
        onClose={() => setNote(null)}
      >
        {book && note !== null ? (
          <NoteForm bookId={book.id} note={note === 'new' ? null : note} onClose={() => setNote(null)} />
        ) : null}
      </DrawerPanel>
    </section>
  );
}
