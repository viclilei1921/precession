import type { Ref } from 'react';
import { useEffect, useImperativeHandle, useLayoutEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import styles from '../read.module.css';
import type { ChapterBlock } from '../scroll';
import { chapterBlocksFromScroller } from '../scroll';

export type ScrollChapter = {
  index: number;
  title: string;
  html: string;
};

export type ReadingScrollHandle = {
  getViewportEl: () => HTMLElement | null;
  getOffset: () => number;
  setOffset: (offset: number) => void;
  measureBlocks: () => ChapterBlock[];
};

export function ScrollColumn({
  chapters,
  total,
  titles,
  loading,
  scrollRef,
  onOffset,
  onClick
}: {
  chapters: ScrollChapter[];
  total: number;
  titles: string[];
  loading: boolean;
  scrollRef: Ref<ReadingScrollHandle | null>;
  onOffset: (offset: number) => void;
  onClick: (event: MouseEvent) => void;
}) {
  const { t } = useTranslation('library');
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const [minHeight, setMinHeight] = useState(0);

  const showPlaceholder = loading && chapters.length === 0;

  useLayoutEffect(() => {
    const viewport = viewportRef.current;
    if (!viewport || showPlaceholder) {
      return;
    }
    const apply = () => {
      const next = viewport.clientHeight;
      setMinHeight((current) => (current === next ? current : next));
    };
    apply();
    const observer = new ResizeObserver(apply);
    observer.observe(viewport);
    return () => observer.disconnect();
  }, [showPlaceholder]);

  useImperativeHandle(
    scrollRef,
    () => ({
      getViewportEl: () => viewportRef.current,
      getOffset: () => viewportRef.current?.scrollTop ?? 0,
      setOffset: (offset) => {
        if (viewportRef.current) {
          viewportRef.current.scrollTop = offset;
        }
      },
      measureBlocks: () => (viewportRef.current ? chapterBlocksFromScroller(viewportRef.current) : [])
    }),
    []
  );

  useEffect(() => {
    const viewport = viewportRef.current;
    if (!viewport || loading || chapters.length === 0) {
      return;
    }
    const onViewportClick = (event: MouseEvent) => onClick(event);
    viewport.addEventListener('click', onViewportClick);
    return () => viewport.removeEventListener('click', onViewportClick);
  }, [chapters.length, loading, onClick]);

  if (showPlaceholder) {
    return (
      <div ref={viewportRef} className={styles.scroll}>
        <p className={styles.muted}>{t('scroll.opening')}</p>
      </div>
    );
  }

  return (
    <div ref={viewportRef} className={styles.scroll} onScroll={(event) => onOffset(event.currentTarget.scrollTop)}>
      {chapters.map((chapter) => {
        const canNext = chapter.index < total - 1;
        const nextTitle = titles[chapter.index + 1]?.trim() || t('reader.nextChapter');
        return (
          <section
            key={chapter.index}
            className={styles.chapterBlock}
            data-chapter={String(chapter.index)}
            style={minHeight > 0 ? { minHeight } : undefined}
          >
            <article
              className={styles.content}
              // biome-ignore lint/security/noDangerouslySetInnerHtml: 阅读命令返回前已去掉脚本、框架和事件
              dangerouslySetInnerHTML={{ __html: chapter.html }}
            />
            <div className={styles.chapterEnd}>
              <p className={styles.chapterEndMark}>{canNext ? t('scroll.chapterEnd') : t('scroll.bookEnd')}</p>
              {canNext ? <p className={styles.chapterEndTitle}>{nextTitle}</p> : null}
              <p className={styles.chapterEndHint}>{canNext ? t('scroll.continueReading') : t('scroll.lastChapter')}</p>
            </div>
          </section>
        );
      })}
    </div>
  );
}
