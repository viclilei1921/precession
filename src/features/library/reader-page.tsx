import { CaretLeftIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useParams } from '@tanstack/react-router';
import { useState } from 'react';
import type { BookNote } from '@/bridge/library';
import { Button } from '@/components/button';
import { RowButton } from '@/components/button/row';
import { Progress } from '@/components/progress';
import styles from '@/components/record.module.css';
import { Scroll } from '@/components/scroll';
import { bookNoteListQuery, bookQuery } from '@/query/library/query';
import { PATH } from '@/router/path';
import { formatMonthDay } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import { BookForm, bookStatuses } from './components/book-form';
import { NoteForm } from './components/note-form';

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
        <Button variant="outlined" onClick={() => void navigate({ to: PATH.library })}>
          <CaretLeftIcon weight="regular" />
          返回书架
        </Button>
        {book ? (
          <Button className={styles.trailing} onClick={() => setEditingBook(true)}>
            编辑这本书
          </Button>
        ) : null}
      </header>
      <Scroll className={styles.scroll}>
        <div className={styles.body}>
          {bookQueryResult.error ? <p className={styles.error}>{errorMessage(bookQueryResult.error)}</p> : null}
          {editingBook && book ? (
            <div className={styles.card}>
              <h2 className={styles.cardTitle}>编辑这本书</h2>
              <BookForm book={book} status={book.status} onClose={() => setEditingBook(false)} />
            </div>
          ) : null}
          {book ? (
            <div className={styles.card}>
              <div className={styles.cardTitle}>
                进度 {Math.round(book.progress * 100)}%
                <span className={styles.extra}>{book.rating != null ? `${book.rating} 分` : '未评分'}</span>
              </div>
              <Progress value={Math.round(book.progress * 100)} tone="library" label="阅读进度" />
            </div>
          ) : null}
          {editingNote !== null ? (
            <div className={styles.card}>
              <h2 className={styles.cardTitle}>{editingNote === 'new' ? '新的书摘或笔记' : '编辑'}</h2>
              <NoteForm
                key={editingNote === 'new' ? 'new' : editingNote.id}
                bookId={bookId}
                note={editingNote === 'new' ? null : editingNote}
                onClose={() => setEditingNote(null)}
              />
            </div>
          ) : null}
          <div className={styles.card}>
            <div className={styles.cardTitle}>
              书摘和笔记
              <Button variant="outlined" onClick={() => setEditingNote('new')}>
                新的一条
              </Button>
            </div>
            {notesQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}
            {!notesQuery.isPending && notes.length === 0 ? <p className={styles.empty}>还没有划线或心得。</p> : null}
            {notes.map((note) => (
              <RowButton key={note.id} onClick={() => setEditingNote(note)}>
                <span className={styles.rowTitle}>
                  {note.kind === 'excerpt' ? '书摘' : '笔记'} · {note.title}
                </span>
                <span className={styles.meta}>
                  {note.chapter ? `${note.chapter} · ` : ''}
                  {formatMonthDay(note.occurredAt)}
                </span>
              </RowButton>
            ))}
          </div>
          <p className={styles.note}>正文阅读器还没有接入。这里先管这本书的档案、书摘和笔记。</p>
        </div>
      </Scroll>
    </section>
  );
}

function statusLabel(status: string): string {
  return bookStatuses.find((item) => item.status === status)?.label ?? '读完';
}
