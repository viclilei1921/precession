import { PlusIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import type { Book, BookNote, BookStatus } from '@/bridge/library';
import { Button } from '@/components/button';
import { SegmentButton } from '@/components/button/segment';
import { Modal } from '@/components/modal';
import { Scroll } from '@/components/scroll';
import { usePageSearch } from '@/hooks/search';
import { bookListQuery, bookNoteListQuery } from '@/query/library/query';
import { PATH } from '@/router/path';
import { useAppStore } from '@/store/app';
import { useSearchStore } from '@/store/search';
import { errorMessage } from '@/utils/error';
import { matchesQuery } from '@/utils/search';
import { bookFile } from './book';
import { AddBook } from './components/add-book';
import { Cover } from './components/cover';
import { noteBody } from './components/note-form';
import styles from './page.module.css';

const SHELVES: { id: BookStatus | 'all'; label: string }[] = [
  { id: 'reading', label: '在读' },
  { id: 'want', label: '想读' },
  { id: 'finished', label: '读完' },
  { id: 'all', label: '全部' }
];

export function LibraryPage() {
  const navigate = useNavigate();
  const search = useSearch({ from: '/library' });
  const booksQuery = useQuery(bookListQuery);
  const [shelf, setShelf] = useState<BookStatus | 'all'>('reading');
  const [adding, setAdding] = useState(false);
  const query = usePageSearch('搜索书库');
  const setQuery = useSearchStore((state) => state.setValue);
  const mobile = useAppStore((state) => state.platform === 'android' || state.platform === 'ios');
  const books = (booksQuery.data ?? []).filter(
    (book) => (shelf === 'all' || book.status === shelf) && matchesQuery(query, book.title, book.author)
  );
  const hero =
    shelf === 'want' || shelf === 'finished'
      ? undefined
      : [...(booksQuery.data ?? [])]
          .filter((book) => book.status === 'reading' && matchesQuery(query, book.title, book.author))
          .sort((left, right) => right.updatedAt - left.updatedAt)[0];

  useEffect(() => {
    if (!search.create) {
      return;
    }
    setAdding(true);
    void navigate({ to: PATH.library, search: {}, replace: true });
  }, [navigate, search.create]);

  function openBook(id: string, readable: boolean) {
    if (readable) {
      void navigate({ to: PATH.bookRead, params: { bookId: id } });
      return;
    }
    void navigate({ to: PATH.bookReader, params: { bookId: id }, search: {} });
  }

  function openDetail(id: string) {
    void navigate({ to: PATH.bookReader, params: { bookId: id }, search: {} });
  }

  return (
    <section className={styles.page}>
      <header className={styles.head} data-mobile={mobile ? 'true' : undefined}>
        {mobile ? (
          <label className={styles.search}>
            <input
              type="search"
              value={query}
              placeholder="搜索书名或作者"
              aria-label="搜索书名或作者"
              onChange={(event) => setQuery(event.target.value)}
            />
          </label>
        ) : null}
        <div className={styles.segments}>
          {SHELVES.map((item) => (
            <SegmentButton key={item.id} selected={shelf === item.id} onClick={() => setShelf(item.id)}>
              {item.label}
            </SegmentButton>
          ))}
        </div>
        <Button className={styles.add} onClick={() => setAdding(true)}>
          <PlusIcon weight="regular" />
          添加书
        </Button>
      </header>
      <Scroll className={styles.scroll}>
        <div className={styles.body}>
          {booksQuery.error ? <p className={styles.empty}>{errorMessage(booksQuery.error)}</p> : null}
          {hero ? <Continue book={hero} /> : null}
          <div className={styles.grid}>
            {books.map((book) => {
              const readable = Boolean(bookFile(book));
              return (
                <div key={book.id} className={styles.card}>
                  <button type="button" className={styles.cardOpen} onClick={() => openBook(book.id, readable)}>
                    <Cover book={book} />
                    <span className={styles.cardTitle}>{book.title}</span>
                    <span className={styles.cardMeta}>
                      {book.author || '佚名'} · {cardMeta(book)}
                    </span>
                  </button>
                  {readable ? (
                    <button type="button" className={styles.detail} onClick={() => openDetail(book.id)}>
                      详情
                    </button>
                  ) : null}
                </div>
              );
            })}
            {!booksQuery.isPending && books.length === 0 ? (
              <p className={styles.empty}>{query.trim() ? '没有匹配的书。' : '这个书架还是空的。'}</p>
            ) : null}
          </div>
        </div>
      </Scroll>
      {adding ? (
        <Modal title="添加书" onClose={() => setAdding(false)}>
          <AddBook onClose={() => setAdding(false)} />
        </Modal>
      ) : null}
    </section>
  );
}

function cardMeta(book: Book) {
  if (book.status === 'finished') {
    return `${'★'.repeat(book.rating ?? 0)}${'☆'.repeat(Math.max(0, 5 - (book.rating ?? 0)))}`;
  }
  if (book.status === 'reading') {
    return `${Math.round(book.progress * 100)}%`;
  }
  return '想读';
}

function Continue({ book }: { book: Book }) {
  const navigate = useNavigate();
  const notesQuery = useQuery(bookNoteListQuery(book.id));
  const quote = (notesQuery.data ?? []).find((note) => highlightQuote(note));
  const percent = Math.round(book.progress * 100);
  const readable = Boolean(bookFile(book));

  return (
    <div className={styles.continue}>
      <button
        type="button"
        className={styles.continueMain}
        onClick={() => {
          if (readable) {
            void navigate({ to: PATH.bookRead, params: { bookId: book.id } });
            return;
          }
          void navigate({ to: PATH.bookReader, params: { bookId: book.id }, search: {} });
        }}
      >
        <Cover book={book} compact />
        <span>
          <p className={styles.kicker}>继续</p>
          <h2>{book.title}</h2>
          <p className={styles.meta}>{book.author}</p>
          {quote ? <p className={styles.quote}>「{noteBody(quote)}」</p> : null}
          <span className={styles.row}>
            <span className={styles.track} role="img" aria-label={`已读 ${percent}%`}>
              <span style={{ width: `${percent}%` }} />
            </span>
            <span className={styles.percent}>{percent}%</span>
          </span>
        </span>
      </button>
      {readable ? (
        <button
          type="button"
          className={styles.detail}
          onClick={() => void navigate({ to: PATH.bookReader, params: { bookId: book.id }, search: {} })}
        >
          详情
        </button>
      ) : null}
    </div>
  );
}

function highlightQuote(note: BookNote) {
  return note.kind === 'excerpt' && note.highlight && !note.locked && (note.body.trim() || note.title.trim());
}
