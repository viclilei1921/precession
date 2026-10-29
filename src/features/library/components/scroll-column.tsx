import type { Ref } from 'react';
import { useEffect, useImperativeHandle, useRef } from 'react';
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
  const viewportRef = useRef<HTMLDivElement | null>(null);

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

  if (loading && chapters.length === 0) {
    return (
      <div ref={viewportRef} className={styles.scroll}>
        <p className={styles.muted}>正在打开…</p>
      </div>
    );
  }

  return (
    <div ref={viewportRef} className={styles.scroll} onScroll={(event) => onOffset(event.currentTarget.scrollTop)}>
      {chapters.map((chapter) => {
        const canNext = chapter.index < total - 1;
        const nextTitle = titles[chapter.index + 1]?.trim() || '下一章';
        return (
          <section key={chapter.index} className={styles.chapterBlock} data-chapter={String(chapter.index)}>
            <article
              className={styles.content}
              // biome-ignore lint/security/noDangerouslySetInnerHtml: 阅读命令返回前已去掉脚本、框架和事件
              dangerouslySetInnerHTML={{ __html: chapter.html }}
            />
            <div className={styles.chapterEnd}>
              <p className={styles.chapterEndMark}>{canNext ? '本章完' : '全书完'}</p>
              {canNext ? <p className={styles.chapterEndTitle}>{nextTitle}</p> : null}
              <p className={styles.chapterEndHint}>{canNext ? '继续滚动阅读下一章' : '已经是最后一章'}</p>
            </div>
          </section>
        );
      })}
    </div>
  );
}
