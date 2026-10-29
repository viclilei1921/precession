import type { QueryClient } from '@tanstack/react-query';
import { queryOptions } from '@tanstack/react-query';
import { bookChapter, bookGet, bookList, bookNoteGet, bookNoteList, bookOutline } from '@/bridge/library';

export const bookListQuery = queryOptions({
  queryKey: ['library', 'books'],
  queryFn: bookList
});

export function bookQuery(id: string) {
  return queryOptions({
    queryKey: ['library', 'book', id],
    queryFn: () => bookGet(id)
  });
}

export function bookNoteListQuery(bookId: string) {
  return queryOptions({
    queryKey: ['library', 'notes', bookId],
    queryFn: () => bookNoteList(bookId)
  });
}

export function bookOutlineQuery(bookId: string) {
  return queryOptions({
    queryKey: ['library', 'outline', bookId],
    queryFn: () => bookOutline(bookId)
  });
}

export function bookChapterQuery(bookId: string, index: number) {
  return queryOptions({
    queryKey: ['library', 'chapter', bookId, index],
    queryFn: () => bookChapter(bookId, index)
  });
}

export function bookNoteQuery(id: string) {
  return queryOptions({
    queryKey: ['library', 'note', id],
    queryFn: () => bookNoteGet(id)
  });
}

export async function refreshLibrary(client: QueryClient) {
  await client.invalidateQueries({ queryKey: ['library'] });
  await client.invalidateQueries({ queryKey: ['timeline'] });
}
