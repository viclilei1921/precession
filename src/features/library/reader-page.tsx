import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate, useParams } from '@tanstack/react-router';
import { useState } from 'react';
import type { BookNote, BookNoteKind } from '@/bridge/library';
import { bookNoteCreate, bookNoteDelete, bookNoteUpdate } from '@/bridge/library';
import { PATH } from '@/router/path';
import { bookNoteListQuery, bookQuery, refreshLibrary } from '@/shared/data/library/query';
import { formatMonthDay, fromDateInputValue, startOfLocalDay, toDateInputValue } from '@/shared/lib/day';
import { errorMessage } from '@/shared/lib/error';
import { Dialog } from '@/shared/ui/dialog';
import styles from '@/shared/ui/record.module.css';
import { BookDialog } from './page';

export function ReaderPage() {
  const { bookId } = useParams({ from: '/library/$bookId' });
  const navigate = useNavigate();
  const bookQueryResult = useQuery(bookQuery(bookId));
  const notesQuery = useQuery(bookNoteListQuery(bookId));
  const [editingBook, setEditingBook] = useState(false);
  const [editingNote, setEditingNote] = useState<BookNote | 'new' | null>(null);
  const book = bookQueryResult.data;
  const notes = [...(notesQuery.data ?? [])].sort((left, right) => right.occurredAt - left.occurredAt);

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>{book?.title ?? '阅读'}</h1>
        <p className={styles.hint}>
          {book ? [book.author, statusLabel(book.status)].filter(Boolean).join(' · ') : '划线和批注会留在这本书上。'}
        </p>
        <button type="button" className={styles.ghost} onClick={() => void navigate({ to: PATH.library })}>
          返回书架
        </button>
        {book ? (
          <button type="button" className={styles.primary} onClick={() => setEditingBook(true)}>
            编辑这本书
          </button>
        ) : null}
      </header>
      {bookQueryResult.error ? <p className={styles.error}>{errorMessage(bookQueryResult.error)}</p> : null}
      {book ? (
        <div className={styles.card}>
          <div className={styles.cardTitle}>
            进度 {Math.round(book.progress * 100)}%
            <span className={styles.extra}>{book.rating != null ? `${book.rating} 分` : '未评分'}</span>
          </div>
          <div className={styles.progress}>
            <i style={{ width: `${Math.round(book.progress * 100)}%` }} />
          </div>
        </div>
      ) : null}
      <div className={styles.card}>
        <div className={styles.cardTitle}>
          书摘和笔记
          <button type="button" className={styles.ghost} onClick={() => setEditingNote('new')}>
            新的一条
          </button>
        </div>
        {notesQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}
        {!notesQuery.isPending && notes.length === 0 ? <p className={styles.empty}>还没有划线或心得。</p> : null}
        {notes.map((note) => (
          <button key={note.id} type="button" className={styles.row} onClick={() => setEditingNote(note)}>
            <span className={styles.rowTitle}>
              {note.kind === 'excerpt' ? '书摘' : '笔记'} · {note.title}
            </span>
            <span className={styles.meta}>
              {note.chapter ? `${note.chapter} · ` : ''}
              {formatMonthDay(note.occurredAt)}
            </span>
          </button>
        ))}
      </div>
      <p className={styles.note}>正文阅读器还没有接入。这里先管这本书的档案、书摘和笔记。</p>
      {editingBook && book ? (
        <BookDialog book={book} status={book.status} onClose={() => setEditingBook(false)} />
      ) : null}
      {editingNote !== null ? (
        <NoteDialog
          key={editingNote === 'new' ? 'new' : editingNote.id}
          bookId={bookId}
          note={editingNote === 'new' ? null : editingNote}
          onClose={() => setEditingNote(null)}
        />
      ) : null}
    </section>
  );
}

function statusLabel(status: string): string {
  if (status === 'want') {
    return '想读';
  }
  if (status === 'reading') {
    return '在读';
  }
  return '读完';
}

function NoteDialog({ bookId, note, onClose }: { bookId: string; note: BookNote | null; onClose: () => void }) {
  const queryClient = useQueryClient();
  const [kind, setKind] = useState<BookNoteKind>(note?.kind ?? 'excerpt');
  const [title, setTitle] = useState(note?.title ?? '');
  const [body, setBody] = useState(note?.body ?? '');
  const [chapter, setChapter] = useState(note?.chapter ?? '');
  const [location, setLocation] = useState(note?.location ?? '');
  const [occurredAt, setOccurredAt] = useState(toDateInputValue(note?.occurredAt ?? startOfLocalDay()));
  const [highlight, setHighlight] = useState(note?.highlight ?? false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const save = useMutation({
    mutationFn: () => {
      const input = {
        bookId,
        kind,
        occurredAt: fromDateInputValue(occurredAt) ?? startOfLocalDay(),
        title,
        body,
        chapter,
        location,
        locked: note?.locked ?? false,
        highlight,
        memberIds: note?.memberIds ?? [],
        tagIds: note?.tagIds ?? [],
        placeIds: note?.placeIds ?? []
      };
      return note ? bookNoteUpdate(note.id, input) : bookNoteCreate(input);
    },
    onSuccess: async () => {
      await refreshLibrary(queryClient);
      onClose();
    }
  });
  const remove = useMutation({
    mutationFn: () => (note ? bookNoteDelete(note.id) : Promise.reject(new Error('记录不存在'))),
    onSuccess: async () => {
      await refreshLibrary(queryClient);
      onClose();
    }
  });
  const pending = save.isPending || remove.isPending;

  return (
    <Dialog title={note ? '编辑' : '新的书摘或笔记'} onClose={onClose}>
      <form
        className={styles.stack}
        onSubmit={(event) => {
          event.preventDefault();
          save.mutate();
        }}
      >
        <div className={styles.pair}>
          <label className={styles.field}>
            类型
            <select value={kind} onChange={(event) => setKind(event.target.value as BookNoteKind)}>
              <option value="excerpt">书摘</option>
              <option value="note">笔记</option>
            </select>
          </label>
          <label className={styles.field}>
            时间
            <input type="date" value={occurredAt} onChange={(event) => setOccurredAt(event.target.value)} />
          </label>
        </div>
        <label className={styles.field}>
          标题
          <input value={title} required onChange={(event) => setTitle(event.target.value)} />
        </label>
        <label className={styles.field}>
          正文
          <textarea value={body} onChange={(event) => setBody(event.target.value)} />
        </label>
        <div className={styles.pair}>
          <label className={styles.field}>
            章节
            <input value={chapter} onChange={(event) => setChapter(event.target.value)} />
          </label>
          <label className={styles.field}>
            位置
            <input value={location} onChange={(event) => setLocation(event.target.value)} />
          </label>
        </div>
        <label className={styles.check}>
          <input type="checkbox" checked={highlight} onChange={(event) => setHighlight(event.target.checked)} />
          标为高光
        </label>
        {save.error ? <p className={styles.error}>{errorMessage(save.error)}</p> : null}
        {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
        <div className={styles.actions}>
          {note ? (
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
