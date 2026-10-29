import { CaretLeftIcon, CaretRightIcon, GearIcon, ListIcon, QuotesIcon, XIcon } from '@phosphor-icons/react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate, useParams } from '@tanstack/react-router';
import type { CSSProperties } from 'react';
import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import type { Book, BookOutlineItem } from '@/bridge/library';
import { bookChapter, bookUpdate } from '@/bridge/library';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { Select } from '@/components/select';
import { Slider } from '@/components/slider';
import { TextField } from '@/components/text-field';
import { bookListQuery, bookOutlineQuery, bookQuery } from '@/query/library/query';
import { PATH } from '@/router/path';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import { locateProgress, progressForScroll, toInput } from './book';
import { NoteForm } from './components/note-form';
import type { ReadingScrollHandle, ScrollChapter } from './components/scroll-column';
import { ScrollColumn } from './components/scroll-column';
import styles from './read.module.css';
import type { ReadingSettings } from './reading-settings';
import {
  FONT_OPTIONS,
  fontFamilyCss,
  GAP_OPTIONS,
  loadReadingSettings,
  paragraphGapCss,
  saveReadingSettings,
  THEME_OPTIONS,
  WIDTH_OPTIONS
} from './reading-settings';
import type { ScrollAnchor } from './scroll';
import {
  captureScrollAnchor,
  chapterScrollPercent,
  chapterWindow,
  restoreScrollTop,
  scrollTopForChapterPercent,
  visibleChapterIndex,
  windowKey
} from './scroll';

type PendingScroll =
  | { kind: 'start'; index: number }
  | { kind: 'end'; index: number }
  | { kind: 'percent'; index: number; percent: number };

function escapeHtml(value: string) {
  return value.replace(/[&<>"']/g, (char) => {
    if (char === '&') {
      return '&amp;';
    }
    if (char === '<') {
      return '&lt;';
    }
    if (char === '>') {
      return '&gt;';
    }
    if (char === '"') {
      return '&quot;';
    }
    return '&#39;';
  });
}

function chapterMarkup(title: string, content: string, html: boolean) {
  if (html) {
    return content;
  }
  const heading = title.trim() ? `<h1>${escapeHtml(title.trim())}</h1>` : '';
  const paragraphs = content
    .split(/\n+/)
    .map((paragraph) => paragraph.trim())
    .filter((paragraph) => paragraph.length > 0)
    .map((paragraph) => `<p>${escapeHtml(paragraph)}</p>`)
    .join('');
  return heading + paragraphs;
}

function formatPercent(ratio: number) {
  return `${Number((Math.min(1, Math.max(0, ratio)) * 100).toFixed(1))}%`;
}

function isInteractiveTarget(target: EventTarget | null) {
  return target instanceof Element && Boolean(target.closest('a, button, input, select, textarea'));
}

export function ReadPage() {
  const { bookId } = useParams({ from: '/library/$bookId/read' });
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const bookQueryResult = useQuery(bookQuery(bookId));
  const outlineQuery = useQuery(bookOutlineQuery(bookId));
  const book = bookQueryResult.data;
  const outline = outlineQuery.data ?? [];
  const [settings, setSettings] = useState<ReadingSettings>(() => loadReadingSettings());
  const [chapters, setChapters] = useState<ScrollChapter[]>([]);
  const [chapterIndex, setChapterIndex] = useState(0);
  const [chapterTitle, setChapterTitle] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [chrome, setChrome] = useState(false);
  const [tocOpen, setTocOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [excerptOpen, setExcerptOpen] = useState(false);
  const [excerptSeed, setExcerptSeed] = useState('');
  const [excerptKey, setExcerptKey] = useState(0);
  const desktop = useAppStore((state) => state.platform !== 'android' && state.platform !== 'ios');
  const selectingRef = useRef(false);
  const [tocSearch, setTocSearch] = useState('');
  const readerRef = useRef<HTMLDivElement | null>(null);
  const scrollRef = useRef<ReadingScrollHandle | null>(null);
  const blocksRef = useRef(scrollRef.current?.measureBlocks() ?? []);
  const bookRef = useRef<Book | null>(null);
  const outlineRef = useRef<BookOutlineItem[]>([]);
  const chapterIndexRef = useRef(0);
  const cacheRef = useRef(new Map<number, ScrollChapter>());
  const pendingRef = useRef<PendingScroll | null>(null);
  const anchorRef = useRef<ScrollAnchor | null>(null);
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const progressFrame = useRef<number | null>(null);
  const bookPercentRef = useRef<HTMLSpanElement | null>(null);
  const chapterPercentRef = useRef<HTMLSpanElement | null>(null);
  const restoringRef = useRef(false);
  chapterIndexRef.current = chapterIndex;
  if (book && bookRef.current?.id !== book.id) {
    bookRef.current = book;
  }
  outlineRef.current = outline;

  const paintWindow = useCallback((index: number) => {
    const total = outlineRef.current.length;
    const next = chapterWindow(index, total)
      .map((item) => cacheRef.current.get(item))
      .filter((item): item is ScrollChapter => Boolean(item));
    setChapters(next);
    setChapterIndex(index);
    setChapterTitle(cacheRef.current.get(index)?.title || outlineRef.current[index]?.title || '');
  }, []);

  const loadWindow = useCallback(
    async (index: number) => {
      const total = outlineRef.current.length;
      const wanted = chapterWindow(index, total);
      await Promise.all(
        wanted.map(async (item) => {
          if (cacheRef.current.has(item)) {
            return;
          }
          const chapter = await bookChapter(bookId, item);
          cacheRef.current.set(item, {
            index: item,
            title: chapter.title,
            html: chapterMarkup(chapter.title, chapter.content, chapter.html)
          });
        })
      );
      paintWindow(index);
    },
    [bookId, paintWindow]
  );

  const goToChapter = useCallback(
    async (index: number, pending?: PendingScroll) => {
      const total = outlineRef.current.length;
      if (total <= 0) {
        return;
      }
      const next = Math.min(total - 1, Math.max(0, index));
      pendingRef.current = pending ?? { kind: 'start', index: next };
      anchorRef.current = null;
      setError(null);
      await loadWindow(next);
    },
    [loadWindow]
  );

  useEffect(() => {
    const current = bookRef.current;
    const list = outlineRef.current;
    if (!book?.id || outline.length === 0 || !current || list.length === 0) {
      return;
    }
    let cancelled = false;
    const place = locateProgress(current.progress, list.length);
    setLoading(true);
    void goToChapter(place.index, { kind: 'percent', index: place.index, percent: place.scroll })
      .catch((reason: unknown) => {
        if (!cancelled) {
          setError(errorMessage(reason));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [book?.id, goToChapter, outline.length]);

  useEffect(() => {
    if (outlineQuery.error) {
      setLoading(false);
      setError(errorMessage(outlineQuery.error));
    }
  }, [outlineQuery.error]);

  // 只在章节窗换掉之后恢复滚动位置。进度绘制只读 ref。
  // biome-ignore lint/correctness/useExhaustiveDependencies: paintProgress 每次渲染都是新函数，不能放进依赖
  useLayoutEffect(() => {
    const handle = scrollRef.current;
    const viewport = handle?.getViewportEl();
    if (!handle || !viewport || chapters.length === 0) {
      return;
    }
    const blocks = handle.measureBlocks();
    blocksRef.current = blocks;
    const viewportHeight = viewport.clientHeight;
    const pending = pendingRef.current;
    restoringRef.current = true;
    if (pending) {
      const block = blocks.find((item) => item.index === pending.index);
      pendingRef.current = null;
      if (block) {
        if (pending.kind === 'start') {
          handle.setOffset(block.top);
        } else if (pending.kind === 'end') {
          handle.setOffset(Math.max(block.top, block.top + block.height - viewportHeight));
        } else {
          handle.setOffset(scrollTopForChapterPercent(viewportHeight, block, pending.percent));
        }
      }
    } else if (anchorRef.current) {
      const top = restoreScrollTop(anchorRef.current, blocks);
      anchorRef.current = null;
      if (top != null) {
        handle.setOffset(top);
      }
    }
    restoringRef.current = false;
    paintProgress();
  }, [chapters]);

  function paintProgress() {
    const handle = scrollRef.current;
    const viewport = handle?.getViewportEl();
    if (!handle || !viewport) {
      return;
    }
    const offset = handle.getOffset();
    const blocks = blocksRef.current;
    const visible = visibleChapterIndex(offset, blocks) ?? chapterIndexRef.current;
    const block = blocks.find((item) => item.index === visible);
    const chapterRatio = block ? chapterScrollPercent(offset, viewport.clientHeight, block) : 0;
    const bookRatio = progressForScroll(visible, chapterRatio, outlineRef.current.length);
    readerRef.current?.style.setProperty('--book-progress', String(bookRatio));
    readerRef.current?.style.setProperty('--chapter-progress', String(chapterRatio));
    if (bookPercentRef.current) {
      bookPercentRef.current.textContent = formatPercent(bookRatio);
    }
    if (chapterPercentRef.current) {
      chapterPercentRef.current.textContent = formatPercent(chapterRatio);
    }
  }

  function schedulePaint() {
    if (progressFrame.current != null) {
      return;
    }
    progressFrame.current = requestAnimationFrame(() => {
      progressFrame.current = null;
      paintProgress();
    });
  }

  function persistProgress() {
    const current = bookRef.current;
    const handle = scrollRef.current;
    const viewport = handle?.getViewportEl();
    if (!current || !handle || !viewport) {
      return;
    }
    const offset = handle.getOffset();
    const blocks = blocksRef.current;
    const visible = visibleChapterIndex(offset, blocks) ?? chapterIndexRef.current;
    const block = blocks.find((item) => item.index === visible);
    const chapterRatio = block ? chapterScrollPercent(offset, viewport.clientHeight, block) : 0;
    const progress = progressForScroll(visible, chapterRatio, outlineRef.current.length);
    if (Math.abs(progress - current.progress) < 0.0001) {
      return;
    }
    const next = { ...current, progress };
    bookRef.current = next;
    queryClient.setQueryData(bookQuery(bookId).queryKey, next);
    queryClient.setQueryData(bookListQuery.queryKey, (list: Book[] | undefined) =>
      list?.map((item) => (item.id === next.id ? { ...item, progress } : item))
    );
    void bookUpdate(current.id, toInput(current, { progress }));
  }

  function onOffset(offset: number) {
    if (restoringRef.current) {
      schedulePaint();
      return;
    }
    const handle = scrollRef.current;
    const total = outlineRef.current.length;
    if (handle && total > 0) {
      const blocks = blocksRef.current;
      const visible = visibleChapterIndex(offset, blocks);
      if (visible != null && visible !== chapterIndexRef.current) {
        const prevWindow = windowKey(chapterWindow(chapterIndexRef.current, total));
        const nextWindow = windowKey(chapterWindow(visible, total));
        if (prevWindow !== nextWindow) {
          anchorRef.current = captureScrollAnchor(offset, blocks);
          paintWindow(visible);
          void loadWindow(visible).catch((reason: unknown) => setError(errorMessage(reason)));
        } else {
          setChapterIndex(visible);
          setChapterTitle(cacheRef.current.get(visible)?.title || outlineRef.current[visible]?.title || '');
        }
      }
    }
    if (saveTimer.current) {
      clearTimeout(saveTimer.current);
    }
    saveTimer.current = setTimeout(persistProgress, 400);
    if (chrome) {
      setChrome(false);
    }
    schedulePaint();
  }

  useEffect(() => {
    return () => {
      if (saveTimer.current) {
        clearTimeout(saveTimer.current);
      }
      if (progressFrame.current != null) {
        cancelAnimationFrame(progressFrame.current);
      }
    };
  }, []);

  useEffect(() => {
    if (!tocOpen && !settingsOpen && !excerptOpen) {
      return;
    }
    const onKey = (event: KeyboardEvent) => {
      if (event.key !== 'Escape') {
        return;
      }
      setTocOpen(false);
      setSettingsOpen(false);
      setExcerptOpen(false);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [excerptOpen, settingsOpen, tocOpen]);

  const tocItems = useMemo(() => {
    const query = tocSearch.trim().toLowerCase();
    if (!query) {
      return outline;
    }
    return outline.filter((item) => item.title.toLowerCase().includes(query));
  }, [outline, tocSearch]);

  function updateSettings(partial: Partial<ReadingSettings>) {
    const next = { ...settings, ...partial };
    saveReadingSettings(next);
    setSettings(next);
  }

  function openExcerpt(text = '') {
    setExcerptSeed(text);
    setExcerptKey((key) => key + 1);
    setExcerptOpen(true);
    setTocOpen(false);
    setSettingsOpen(false);
    setChrome(false);
  }

  function closeDrawers() {
    setTocOpen(false);
    setSettingsOpen(false);
    setExcerptOpen(false);
  }

  useEffect(() => {
    if (!desktop) {
      return;
    }
    const onUp = () => {
      const selection = window.getSelection();
      const text = selection?.toString().trim() ?? '';
      const node = selection?.anchorNode;
      const viewport = scrollRef.current?.getViewportEl();
      if (!text || !node || !viewport?.contains(node)) {
        return;
      }
      selectingRef.current = true;
      selection?.removeAllRanges();
      setExcerptSeed(text);
      setExcerptKey((key) => key + 1);
      setExcerptOpen(true);
      setTocOpen(false);
      setSettingsOpen(false);
      setChrome(false);
    };
    window.addEventListener('mouseup', onUp);
    return () => window.removeEventListener('mouseup', onUp);
  }, [desktop]);

  function onReaderClick(event: MouseEvent) {
    if (selectingRef.current) {
      selectingRef.current = false;
      return;
    }
    if (tocOpen || settingsOpen || excerptOpen || isInteractiveTarget(event.target)) {
      return;
    }
    if (window.getSelection()?.toString()) {
      return;
    }
    setChrome((open) => !open);
  }

  const readerStyle = {
    '--reader-font-size': `${settings.fontSize}px`,
    '--reader-line-height': String(settings.lineHeight),
    '--reader-max-width': `${settings.pageMaxWidth}%`,
    '--reader-font': fontFamilyCss(settings.fontFamily),
    '--reader-gap': paragraphGapCss(settings.paragraphGap)
  } as CSSProperties;
  const titles = outline.map((item) => item.title);
  const canPrev = chapterIndex > 0;
  const canNext = chapterIndex < outline.length - 1;

  return (
    <section
      ref={readerRef}
      className={styles.reader}
      data-theme={settings.readerTheme}
      data-chrome={chrome ? 'true' : 'false'}
      data-selectable={desktop ? 'true' : undefined}
      style={readerStyle}
    >
      <header className={styles.toolbar}>
        <div className={styles.toolbarRow}>
          <Button variant="outlined" onClick={() => void navigate({ to: PATH.library, search: {} })}>
            <CaretLeftIcon weight="regular" />
            返回
          </Button>
          <div className={styles.toolbarTitle}>
            <strong>{book?.title ?? '阅读'}</strong>
            <span>{chapterTitle || outline[chapterIndex]?.title}</span>
          </div>
          <div className={styles.percents}>
            <span>
              全书 <span ref={bookPercentRef}>0%</span>
            </span>
            <span>
              本章 <span ref={chapterPercentRef}>0%</span>
            </span>
          </div>
        </div>
        <div className={styles.tracks}>
          <div className={styles.track} role="progressbar" aria-label="全书进度">
            <span className={styles.fill} />
          </div>
          <div className={styles.track} role="progressbar" aria-label="本章进度">
            <span className={`${styles.fill} ${styles.fillChapter}`} />
          </div>
        </div>
      </header>
      {error ? <p className={styles.error}>{error}</p> : null}
      <div className={styles.body}>
        {outlineQuery.isSuccess && outline.length === 0 && !error ? (
          <p className={styles.muted}>没有可阅读的章节。</p>
        ) : (
          <ScrollColumn
            chapters={chapters}
            total={outline.length}
            titles={titles}
            loading={loading && !error}
            scrollRef={scrollRef}
            onOffset={onOffset}
            onClick={onReaderClick}
          />
        )}
      </div>
      <nav className={styles.dock} aria-label="阅读工具">
        <Button variant="outlined" disabled={!canPrev} onClick={() => void goToChapter(chapterIndex - 1)}>
          <CaretLeftIcon weight="regular" />
          上一章
        </Button>
        <Button
          onClick={() => {
            setChrome(false);
            setSettingsOpen(false);
            setExcerptOpen(false);
            setTocOpen(true);
          }}
        >
          <ListIcon weight="regular" />
          目录
        </Button>
        <Button variant="outlined" disabled={!canNext} onClick={() => void goToChapter(chapterIndex + 1)}>
          <CaretRightIcon weight="regular" />
          下一章
        </Button>
        <Button variant="outlined" onClick={() => openExcerpt()}>
          <QuotesIcon weight="regular" />
          书摘
        </Button>
        <Button
          variant="outlined"
          onClick={() => {
            setChrome(false);
            setTocOpen(false);
            setExcerptOpen(false);
            setSettingsOpen(true);
          }}
        >
          <GearIcon weight="regular" />
          设置
        </Button>
      </nav>
      <button
        type="button"
        className={styles.scrim}
        data-open={tocOpen || settingsOpen || excerptOpen ? 'true' : undefined}
        tabIndex={tocOpen || settingsOpen || excerptOpen ? 0 : -1}
        aria-hidden={tocOpen || settingsOpen || excerptOpen ? undefined : true}
        aria-label="关闭"
        onClick={closeDrawers}
      />
      <aside
        className={`${styles.drawer} ${styles.drawerLeft}`}
        data-open={tocOpen ? 'true' : undefined}
        aria-label="目录"
      >
        <header className={styles.drawerHead}>
          <h2>目录</h2>
          <IconButton label="关闭目录" onClick={() => setTocOpen(false)}>
            <XIcon weight="regular" />
          </IconButton>
        </header>
        <div className={styles.drawerBody}>
          <TextField label="搜索章节" value={tocSearch} onChange={(event) => setTocSearch(event.target.value)} />
          <nav className={styles.toc}>
            {tocItems.map((item) => (
              <button
                key={item.index}
                type="button"
                className={styles.tocItem}
                data-on={item.index === chapterIndex ? 'true' : undefined}
                onClick={() => {
                  void goToChapter(item.index);
                  setTocOpen(false);
                }}
              >
                {item.title || `第 ${item.index + 1} 章`}
              </button>
            ))}
          </nav>
        </div>
      </aside>
      <aside
        className={`${styles.drawer} ${styles.drawerRight}`}
        data-open={settingsOpen ? 'true' : undefined}
        aria-label="阅读设置"
      >
        <header className={styles.drawerHead}>
          <h2>阅读设置</h2>
          <IconButton label="关闭阅读设置" onClick={() => setSettingsOpen(false)}>
            <XIcon weight="regular" />
          </IconButton>
        </header>
        <div className={styles.settings}>
          <Slider
            label="字号"
            min={16}
            max={40}
            value={settings.fontSize}
            onChange={(fontSize) => updateSettings({ fontSize })}
          />
          <Select
            label="字体"
            value={settings.fontFamily}
            onChange={(event) => updateSettings({ fontFamily: event.target.value as ReadingSettings['fontFamily'] })}
          >
            {FONT_OPTIONS.map((item) => (
              <option key={item.value} value={item.value}>
                {item.label}
              </option>
            ))}
          </Select>
          <Select
            label="主题"
            value={settings.readerTheme}
            onChange={(event) => updateSettings({ readerTheme: event.target.value as ReadingSettings['readerTheme'] })}
          >
            {THEME_OPTIONS.map((item) => (
              <option key={item.value} value={item.value}>
                {item.label}
              </option>
            ))}
          </Select>
          <Select
            label="版心"
            value={String(settings.pageMaxWidth)}
            onChange={(event) =>
              updateSettings({ pageMaxWidth: Number(event.target.value) as ReadingSettings['pageMaxWidth'] })
            }
          >
            {WIDTH_OPTIONS.map((item) => (
              <option key={item.value} value={item.value}>
                {item.label}
              </option>
            ))}
          </Select>
          <Select
            label="段距"
            value={settings.paragraphGap}
            onChange={(event) =>
              updateSettings({ paragraphGap: event.target.value as ReadingSettings['paragraphGap'] })
            }
          >
            {GAP_OPTIONS.map((item) => (
              <option key={item.value} value={item.value}>
                {item.label}
              </option>
            ))}
          </Select>
        </div>
      </aside>
      <aside
        className={`${styles.drawer} ${styles.drawerRight}`}
        data-open={excerptOpen ? 'true' : undefined}
        aria-label="书摘"
      >
        <header className={styles.drawerHead}>
          <h2>书摘</h2>
          <IconButton label="关闭书摘" onClick={() => setExcerptOpen(false)}>
            <XIcon weight="regular" />
          </IconButton>
        </header>
        {excerptOpen ? (
          <NoteForm
            key={excerptKey}
            bookId={bookId}
            note={null}
            lockKind
            preset={{
              kind: 'excerpt',
              chapter: chapterTitle || outline[chapterIndex]?.title || '',
              body: excerptSeed
            }}
            onClose={() => setExcerptOpen(false)}
          />
        ) : null}
      </aside>
    </section>
  );
}
