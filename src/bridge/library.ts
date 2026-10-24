import { invokeCommand } from './invoke';
import type { Upload } from './upload';

/** 书架状态 */
export type BookStatus = 'want' | 'reading' | 'finished';

/** 书摘或读书笔记 */
export type BookNoteKind = 'excerpt' | 'note';

/** 一本书 */
export type Book = {
  id: string;
  title: string;
  author: string;
  coverPath: string;
  status: BookStatus;
  progress: number;
  rating: number | null;
  startedAt: number | null;
  finishedAt: number | null;
  createdAt: number;
  updatedAt: number;
  /** 挂在这本书上的文件 */
  uploads: Upload[];
};

/** 新建或修改书 */
export type BookInput = {
  title: string;
  author: string;
  coverPath: string;
  status: BookStatus;
  progress: number;
  rating: number | null;
  startedAt: number | null;
  finishedAt: number | null;
};

/** 书摘或读书笔记 */
export type BookNote = {
  id: string;
  bookId: string;
  kind: BookNoteKind;
  occurredAt: number;
  title: string;
  body: string;
  chapter: string;
  location: string;
  locked: boolean;
  highlight: boolean;
  memberIds: string[];
  tagIds: string[];
  placeIds: string[];
  createdAt: number;
  updatedAt: number;
};

/** 新建或修改书摘、读书笔记 */
export type BookNoteInput = {
  bookId: string;
  kind: BookNoteKind;
  occurredAt: number;
  title: string;
  body: string;
  chapter: string;
  location: string;
  locked: boolean;
  highlight: boolean;
  memberIds: string[];
  tagIds: string[];
  placeIds: string[];
};

/** 列出未删除的书 */
export function bookList() {
  return invokeCommand<Book[]>('book_list');
}

/** 读取一本书 */
export function bookGet(id: string) {
  return invokeCommand<Book>('book_get', { id });
}

/** 添加书 */
export function bookCreate(input: BookInput) {
  return invokeCommand<Book>('book_create', { input });
}

/** 修改书 */
export function bookUpdate(id: string, input: BookInput) {
  return invokeCommand<Book>('book_update', { id, input });
}

/** 软删除书 */
export function bookDelete(id: string) {
  return invokeCommand<void>('book_delete', { id });
}

/** 列出一本书的书摘和笔记 */
export function bookNoteList(bookId: string) {
  return invokeCommand<BookNote[]>('book_note_list', { bookId });
}

/** 读取一条书摘或笔记 */
export function bookNoteGet(id: string) {
  return invokeCommand<BookNote>('book_note_get', { id });
}

/** 新建书摘或笔记 */
export function bookNoteCreate(input: BookNoteInput) {
  return invokeCommand<BookNote>('book_note_create', { input });
}

/** 修改书摘或笔记 */
export function bookNoteUpdate(id: string, input: BookNoteInput) {
  return invokeCommand<BookNote>('book_note_update', { id, input });
}

/** 软删除书摘或笔记 */
export function bookNoteDelete(id: string) {
  return invokeCommand<void>('book_note_delete', { id });
}

/** 导入前从本地文件读出的书名和作者 */
export type BookPeek = {
  title: string;
  author: string;
  /** epub、mobi 或 txt */
  format: string;
};

/** 目录里的一章 */
export type BookOutlineItem = {
  index: number;
  title: string;
};

/** 一章正文。html 为真时 content 是清理过的 HTML */
export type BookChapter = {
  index: number;
  title: string;
  content: string;
  html: boolean;
};

/** 从用户选中的文件读出书名、作者和格式 */
export function bookPeek(path: string) {
  return invokeCommand<BookPeek>('book_peek', { path });
}

/** 列出这本书的目录 */
export function bookOutline(bookId: string) {
  return invokeCommand<BookOutlineItem[]>('book_outline', { bookId });
}

/** 读取一章正文 */
export function bookChapter(bookId: string, index: number) {
  return invokeCommand<BookChapter>('book_chapter', { bookId, index });
}
