export type ChapterBlock = {
  index: number;
  top: number;
  height: number;
};

export type ScrollAnchor = {
  index: number;
  localOffset: number;
};

function clamp01(value: number) {
  if (!Number.isFinite(value)) {
    return 0;
  }
  return Math.min(1, Math.max(0, value));
}

/** 当前章和相邻章，最多三章 */
export function chapterWindow(current: number, total: number) {
  if (total <= 0) {
    return [];
  }
  const index = Math.min(Math.max(0, current), total - 1);
  const window: number[] = [];
  if (index > 0) {
    window.push(index - 1);
  }
  window.push(index);
  if (index < total - 1) {
    window.push(index + 1);
  }
  return window;
}

export function windowKey(indices: number[]) {
  return indices.join(',');
}

/** 视口顶部附近落在哪一章 */
export function visibleChapterIndex(scrollTop: number, blocks: ChapterBlock[], probeOffset = 48) {
  if (blocks.length === 0) {
    return null;
  }
  const probe = scrollTop + probeOffset;
  for (const block of blocks) {
    if (probe >= block.top && probe < block.top + block.height) {
      return block.index;
    }
  }
  if (probe < blocks[0].top) {
    return blocks[0].index;
  }
  return blocks[blocks.length - 1].index;
}

/** 本章已经滚过的比例 */
export function chapterScrollPercent(scrollTop: number, viewportHeight: number, block: ChapterBlock) {
  if (block.height <= viewportHeight) {
    return 1;
  }
  return clamp01((scrollTop - block.top) / (block.height - viewportHeight));
}

export function scrollTopForChapterPercent(viewportHeight: number, block: ChapterBlock, scrollPercent: number) {
  if (block.height <= viewportHeight) {
    return block.top;
  }
  return block.top + clamp01(scrollPercent) * (block.height - viewportHeight);
}

export function captureScrollAnchor(scrollTop: number, blocks: ChapterBlock[]): ScrollAnchor | null {
  const index = visibleChapterIndex(scrollTop, blocks);
  const block = blocks.find((item) => item.index === index);
  if (!block) {
    return null;
  }
  return { index: block.index, localOffset: scrollTop - block.top };
}

export function restoreScrollTop(anchor: ScrollAnchor, blocks: ChapterBlock[]) {
  const block = blocks.find((item) => item.index === anchor.index);
  if (!block) {
    return null;
  }
  return block.top + anchor.localOffset;
}

export function chapterBlocksFromScroller(scroller: HTMLElement): ChapterBlock[] {
  const blocks: ChapterBlock[] = [];
  for (const node of scroller.querySelectorAll('[data-chapter]')) {
    if (!(node instanceof HTMLElement)) {
      continue;
    }
    const index = Number(node.dataset.chapter);
    if (!Number.isFinite(index)) {
      continue;
    }
    blocks.push({ index, top: node.offsetTop, height: node.offsetHeight });
  }
  return blocks;
}
