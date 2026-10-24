import type { Book, BookInput, BookStatus } from '@/bridge/library';
import type { Upload } from '@/bridge/upload';

export const bookStatusLabel: Record<BookStatus, string> = {
  want: '想读',
  reading: '在读',
  finished: '读完'
};

/** 用现有字段拼出更新入参，读完日期原样带回 */
export function toInput(book: Book, patch: Partial<BookInput> = {}): BookInput {
  return {
    title: book.title,
    author: book.author,
    coverPath: book.coverPath,
    status: book.status,
    progress: book.progress,
    rating: book.rating,
    startedAt: book.startedAt,
    finishedAt: book.finishedAt,
    ...patch
  };
}

/** 这本书上最新的未加密书籍文件 */
export function bookFile(book: Book): Upload | undefined {
  return [...book.uploads].reverse().find((item) => item.kind === 'book' && !item.encrypted);
}

/** 有书籍文件，但全部已加密 */
export function bookEncrypted(book: Book) {
  const files = book.uploads.filter((item) => item.kind === 'book');
  return files.length > 0 && files.every((item) => item.encrypted);
}

/** 封面角标。azw 和 azw3 按 MOBI 显示 */
export function bookFormat(book: Book): string {
  const file = bookFile(book) ?? [...book.uploads].reverse().find((item) => item.kind === 'book');
  if (!file) {
    return '';
  }
  const ext = file.name.split('.').pop()?.toLowerCase() ?? '';
  if (ext === 'epub') {
    return 'EPUB';
  }
  if (ext === 'txt') {
    return 'TXT';
  }
  if (ext === 'mobi' || ext === 'azw' || ext === 'azw3') {
    return 'MOBI';
  }
  return ext.toUpperCase();
}

/** 从 0 到 1 的进度还原章节和章内位置。整书进度 = (章序号 + 章内滚动) / 总章数 */
export function locateProgress(progress: number, count: number) {
  if (count <= 0) {
    return { index: 0, scroll: 0 };
  }
  const scaled = Math.min(count, Math.max(0, progress) * count);
  if (scaled >= count) {
    return { index: count - 1, scroll: 1 };
  }
  const index = Math.floor(scaled);
  return { index, scroll: scaled - index };
}

/** 把当前章和章内滚动写回 0 到 1 的进度 */
export function progressForScroll(index: number, scroll: number, count: number) {
  if (count <= 0) {
    return 0;
  }
  const within = Math.min(1, Math.max(0, scroll));
  return Math.min(1, (index + within) / count);
}
