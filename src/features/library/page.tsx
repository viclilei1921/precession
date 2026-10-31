import { MagnifyingGlassIcon, PlusIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import type { Book, BookStatus } from '@/bridge/library';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { SegmentButton } from '@/components/button/segment';
import { Scroll } from '@/components/scroll';
import { DrawerPanel, DrawerScrim } from '@/features/plan/components/drawer-frame';
import { usePageSearch } from '@/hooks/search';
import { Fab } from '@/layout/mobile/fab';
import { bookListQuery } from '@/query/library/query';
import { PATH } from '@/router/path';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import { matchesQuery } from '@/utils/search';
import { bookFile, displayTitle } from './book';
import { AddBook } from './components/add-book';
import { Cover } from './components/cover';
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
  const [finding, setFinding] = useState(false);
  const [localQuery, setLocalQuery] = useState('');
  const pageQuery = usePageSearch('搜索书库');
  const mobile = useAppStore((state) => state.isMobile());
  const query = mobile ? localQuery : pageQuery;
  const books = (booksQuery.data ?? []).filter(
    (book) =>
      (shelf === 'all' || book.status === shelf) &&
      matchesQuery(query, book.title, displayTitle(book.title), book.author)
  );

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
      <header className={styles.head}>
        <div className={styles.segments} role="tablist" aria-label="书架">
          {SHELVES.map((item) => (
            <SegmentButton key={item.id} selected={shelf === item.id} onClick={() => setShelf(item.id)}>
              {item.label}
            </SegmentButton>
          ))}
        </div>
        {mobile ? (
          <IconButton
            className={styles.add}
            label="搜索书库"
            onClick={() =>
              setFinding((current) => {
                if (current) {
                  setLocalQuery('');
                }
                return !current;
              })
            }
          >
            <MagnifyingGlassIcon weight="regular" />
          </IconButton>
        ) : (
          <Button className={styles.add} onClick={() => setAdding(true)}>
            <PlusIcon weight="regular" />
            添加书
          </Button>
        )}
      </header>
      {mobile && finding ? (
        <form
          className={styles.search}
          onSubmit={(event) => {
            event.preventDefault();
          }}
        >
          <input
            type="search"
            value={localQuery}
            placeholder="搜索书名或作者"
            aria-label="搜索书库"
            onChange={(event) => setLocalQuery(event.target.value)}
          />
          <button
            type="button"
            onClick={() => {
              setLocalQuery('');
              setFinding(false);
            }}
          >
            取消
          </button>
        </form>
      ) : null}
      <Scroll className={styles.scroll}>
        <div className={styles.body}>
          {booksQuery.error ? <p className={styles.empty}>{errorMessage(booksQuery.error)}</p> : null}
          <div className={styles.grid}>
            {books.map((book) => {
              const readable = Boolean(bookFile(book));
              return (
                <div key={book.id} className={styles.card}>
                  <button type="button" className={styles.cardOpen} onClick={() => openBook(book.id, readable)}>
                    <Cover book={book} />
                    <span className={styles.cardTitle}>{displayTitle(book.title)}</span>
                    <span className={styles.cardMeta}>
                      {book.author || '佚名'} · {cardMeta(book)}
                    </span>
                  </button>
                  {readable ? (
                    <Button variant="text" className={styles.detail} onClick={() => openDetail(book.id)}>
                      详情
                    </Button>
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
      {mobile && !adding ? <Fab label="添加书" onClick={() => setAdding(true)} /> : null}
      <DrawerScrim open={adding} onClose={() => setAdding(false)} />
      <DrawerPanel open={adding} title="添加书" label="添加书" onClose={() => setAdding(false)}>
        {adding ? <AddBook onClose={() => setAdding(false)} /> : null}
      </DrawerPanel>
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
