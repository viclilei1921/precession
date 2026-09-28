import { invokeCommand } from './invoke';

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
