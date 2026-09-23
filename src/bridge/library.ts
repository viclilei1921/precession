import { invokeCommand } from './invoke';

/** 书架状态 */
export type BookStatus = 'want' | 'reading' | 'done';

/** 书摘或读书笔记 */
export type QuoteType = 'excerpt' | 'note';

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
export type BookWrite = {
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
export type QuoteItem = {
  id: string;
  bookId: string;
  type: QuoteType;
  occurredAt: number;
  title: string;
  body: string;
  chapter: string;
  location: string;
  createdAt: number;
  updatedAt: number;
};

/** 新建或修改书摘、读书笔记 */
export type QuoteWrite = {
  bookId: string;
  type: QuoteType;
  occurredAt: number;
  title: string;
  body: string;
  chapter: string;
  location: string;
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
export function bookCreate(input: BookWrite) {
  return invokeCommand<Book>('book_create', { input });
}

/** 修改书 */
export function bookUpdate(id: string, input: BookWrite) {
  return invokeCommand<Book>('book_update', { id, input });
}

/** 软删除书 */
export function bookDelete(id: string) {
  return invokeCommand<void>('book_delete', { id });
}

/** 列出一本书的书摘和笔记 */
export function quoteList(bookId: string) {
  return invokeCommand<QuoteItem[]>('quote_list', { bookId });
}

/** 读取一条书摘或笔记 */
export function quoteGet(id: string) {
  return invokeCommand<QuoteItem>('quote_get', { id });
}

/** 新建书摘或笔记 */
export function quoteCreate(input: QuoteWrite) {
  return invokeCommand<QuoteItem>('quote_create', { input });
}

/** 修改书摘或笔记 */
export function quoteUpdate(id: string, input: QuoteWrite) {
  return invokeCommand<QuoteItem>('quote_update', { id, input });
}

/** 软删除书摘或笔记 */
export function quoteDelete(id: string) {
  return invokeCommand<void>('quote_delete', { id });
}
