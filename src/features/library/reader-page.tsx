import { CaretLeftIcon } from '@phosphor-icons/react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate, useParams, useSearch } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { pickFiles } from '@/bridge/files';
import type { Book, BookNote, BookStatus } from '@/bridge/library';
import { bookDelete, bookUpdate } from '@/bridge/library';
import { onTaskUpdate, taskEnqueue } from '@/bridge/task';
import { Button } from '@/components/button';
import { SegmentButton } from '@/components/button/segment';
import { Scroll } from '@/components/scroll';
import { DrawerPanel, DrawerScrim } from '@/features/plan/components/drawer-frame';
import { performMobileBack, useBackLayer } from '@/layout/mobile/back';
import { bookNoteListQuery, bookQuery, refreshLibrary } from '@/query/library/query';
import { PATH } from '@/router/path';
import { useAppStore } from '@/store/app';
import { formatMonthDay } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import { bookEncrypted, bookFile, bookStatusLabel, displayTitle, toInput } from './book';
import styles from './book.module.css';
import { Cover } from './components/cover';
import { NoteForm, noteBody } from './components/note-form';

const STATUSES: BookStatus[] = ['reading', 'want', 'finished'];
const EXTENSIONS = ['epub', 'mobi', 'txt', 'azw', 'azw3'];

export function ReaderPage() {
  const { t } = useTranslation('library');
  const { bookId } = useParams({ from: '/library/$bookId' });
  const search = useSearch({ from: '/library/$bookId' });
  const navigate = useNavigate();
  const mobile = useAppStore((state) => state.isMobile());
  const queryClient = useQueryClient();
  const bookQueryResult = useQuery(bookQuery(bookId));
  const notesQuery = useQuery(bookNoteListQuery(bookId));
  const [note, setNote] = useState<BookNote | null | 'new'>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  useBackLayer(note !== null, () => setNote(null));
  useBackLayer(confirmDelete, () => setConfirmDelete(false));
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
      void navigate({ to: PATH.library, replace: true });
    }
  });
  const attach = useMutation({
    mutationFn: async () => {
      const files = await pickFiles({ multiple: false, extensions: EXTENSIONS });
      const source = files[0];
      if (!source) {
        return null;
      }
      const task = await taskEnqueue({
        kind: 'importUpload',
        source,
        owner: 'book',
        ownerId: bookId,
        uploadKind: 'book',
        encrypt: false,
        password: ''
      });
      return task.id;
    },
    onSuccess: (taskId) => {
      if (!taskId) {
        return;
      }
      void navigate({ to: PATH.bookReader, params: { bookId }, search: { task: taskId } });
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
          void navigate({
            to: PATH.bookReader,
            params: { bookId },
            search: search.note ? { note: search.note } : {},
            replace: true
          });
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
  }, [bookId, navigate, queryClient, search.note, search.task]);

  useEffect(() => {
    if (!search.task || !file) {
      return;
    }
    void navigate({
      to: PATH.bookReader,
      params: { bookId },
      search: search.note ? { note: search.note } : {},
      replace: true
    });
  }, [bookId, file, navigate, search.note, search.task]);

  useEffect(() => {
    if (!search.note || notesQuery.isPending) {
      return;
    }
    const found = notesQuery.data?.find((item) => item.id === search.note);
    if (found) {
      setNote(found);
    }
    void navigate({
      to: PATH.bookReader,
      params: { bookId },
      search: search.task ? { task: search.task } : {},
      replace: true
    });
  }, [bookId, navigate, notesQuery.data, notesQuery.isPending, search.note, search.task]);

  function patch(partial: Partial<Book>) {
    if (!book) {
      return;
    }
    update.mutate({ ...book, ...partial });
  }

  return (
    <section className={styles.page}>
      <Button
        variant="outlined"
        onClick={() => {
          if (mobile) {
            performMobileBack();
            return;
          }
          void navigate({ to: PATH.library });
        }}
      >
        <CaretLeftIcon weight="regular" />
        {t('detail.backToShelf')}
      </Button>
      <Scroll className={styles.scroll}>
        {bookQueryResult.error ? <p className={styles.note}>{errorMessage(bookQueryResult.error)}</p> : null}
        {book ? (
          <div className={styles.layout}>
            <div className={styles.dossier}>
              <Cover book={book} />
              <h1>{displayTitle(book.title)}</h1>
              <p className={styles.meta}>{book.author || t('page.anonymous')}</p>
              <div className={styles.segments}>
                {STATUSES.map((status) => (
                  <SegmentButton key={status} selected={book.status === status} onClick={() => patch({ status })}>
                    {bookStatusLabel(status)}
                  </SegmentButton>
                ))}
              </div>
              <div className={styles.stars}>
                {[1, 2, 3, 4, 5].map((star) => (
                  <button
                    key={star}
                    type="button"
                    data-on={(book.rating ?? 0) >= star ? 'true' : undefined}
                    aria-label={t('detail.starAria', { count: star })}
                    onClick={() => patch({ rating: book.rating === star ? null : star })}
                  >
                    ★
                  </button>
                ))}
              </div>
              <label className={styles.range}>
                {t('detail.progress', { percent: Math.round(book.progress * 100) })}
                <input
                  type="range"
                  min={0}
                  max={100}
                  value={Math.round(book.progress * 100)}
                  aria-label={t('detail.progressAria')}
                  onChange={(event) => patch({ progress: Number(event.target.value) / 100 })}
                />
              </label>
              {importing ? <p className={styles.note}>{t('detail.importing')}</p> : null}
              {file || importing ? null : (
                <p className={styles.note}>{bookEncrypted(book) ? t('detail.encrypted') : t('detail.noContent')}</p>
              )}
              <div className={styles.actions}>
                {file ? (
                  <Button
                    className={styles.action}
                    onClick={() => void navigate({ to: PATH.bookRead, params: { bookId } })}
                  >
                    {t('detail.continueReading')}
                  </Button>
                ) : null}
                {!file && !importing && !bookEncrypted(book) ? (
                  <Button className={styles.action} disabled={attach.isPending} onClick={() => attach.mutate()}>
                    {t('detail.importText')}
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
                  {confirmDelete ? t('detail.confirmDelete') : t('detail.deleteBook')}
                </Button>
              </div>
              {update.error ? <p className={styles.error}>{errorMessage(update.error)}</p> : null}
              {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
              {attach.error ? <p className={styles.error}>{errorMessage(attach.error)}</p> : null}
            </div>
            <div className={styles.notes}>
              <div className={styles.notesHead}>
                <h2>{t('detail.notesTitle')}</h2>
                <Button onClick={() => setNote('new')}>{t('detail.newNote')}</Button>
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
                      <span>{item.kind === 'excerpt' ? t('detail.excerpt') : t('detail.note')}</span>
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
                    <p>{t('detail.emptyNotesTitle')}</p>
                    <span>{t('detail.emptyNotesHint')}</span>
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
        title={note && note !== 'new' ? t('detail.editNote') : t('detail.newNote')}
        label={t('detail.noteDrawerLabel')}
        onClose={() => setNote(null)}
      >
        {book && note !== null ? (
          <NoteForm bookId={book.id} note={note === 'new' ? null : note} onClose={() => setNote(null)} />
        ) : null}
      </DrawerPanel>
    </section>
  );
}
