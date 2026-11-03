import { MagnifyingGlassIcon, PlusIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { Book, BookStatus } from '@/bridge/library';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { SegmentButton } from '@/components/button/segment';
import { Scroll } from '@/components/scroll';
import { DrawerPanel, DrawerScrim } from '@/features/plan/components/drawer-frame';
import { usePageSearch } from '@/hooks/search';
import { useBackLayer } from '@/layout/mobile/back';
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

const SHELVES: { id: BookStatus | 'all'; key: string }[] = [
  { id: 'reading', key: 'page.shelves.reading' },
  { id: 'want', key: 'page.shelves.want' },
  { id: 'finished', key: 'page.shelves.finished' },
  { id: 'all', key: 'page.shelves.all' }
];

export function LibraryPage() {
  const { t } = useTranslation(['library', 'common']);
  const navigate = useNavigate();
  const search = useSearch({ from: '/library' });
  const booksQuery = useQuery(bookListQuery);
  const [shelf, setShelf] = useState<BookStatus | 'all'>('reading');
  const [adding, setAdding] = useState(false);
  const [finding, setFinding] = useState(false);
  const [localQuery, setLocalQuery] = useState('');
  const pageQuery = usePageSearch(t('page.searchPage'));
  const mobile = useAppStore((state) => state.isMobile());
  useBackLayer(adding, () => setAdding(false));
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
        <div className={styles.segments} role="tablist" aria-label={t('page.shelfAria')}>
          {SHELVES.map((item) => (
            <SegmentButton key={item.id} selected={shelf === item.id} onClick={() => setShelf(item.id)}>
              {t(item.key)}
            </SegmentButton>
          ))}
        </div>
        {mobile ? (
          <IconButton
            className={styles.add}
            label={t('page.searchAria')}
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
            {t('page.add')}
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
            placeholder={t('page.searchPlaceholder')}
            aria-label={t('page.searchAria')}
            onChange={(event) => setLocalQuery(event.target.value)}
          />
          <button
            type="button"
            onClick={() => {
              setLocalQuery('');
              setFinding(false);
            }}
          >
            {t('common:button.cancel')}
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
                      {book.author || t('page.anonymous')} · {cardMeta(book, t('status.want'))}
                    </span>
                  </button>
                  {readable ? (
                    <Button variant="text" className={styles.detail} onClick={() => openDetail(book.id)}>
                      {t('page.detail')}
                    </Button>
                  ) : null}
                </div>
              );
            })}
            {!booksQuery.isPending && books.length === 0 ? (
              <p className={styles.empty}>{query.trim() ? t('page.emptySearch') : t('page.emptyShelf')}</p>
            ) : null}
          </div>
        </div>
      </Scroll>
      {mobile && !adding ? <Fab label={t('page.add')} onClick={() => setAdding(true)} /> : null}
      <DrawerScrim open={adding} onClose={() => setAdding(false)} />
      <DrawerPanel open={adding} title={t('page.add')} label={t('page.add')} onClose={() => setAdding(false)}>
        {adding ? <AddBook onClose={() => setAdding(false)} /> : null}
      </DrawerPanel>
    </section>
  );
}

function cardMeta(book: Book, wantLabel: string) {
  if (book.status === 'finished') {
    return `${'★'.repeat(book.rating ?? 0)}${'☆'.repeat(Math.max(0, 5 - (book.rating ?? 0)))}`;
  }
  if (book.status === 'reading') {
    return `${Math.round(book.progress * 100)}%`;
  }
  return wantLabel;
}
