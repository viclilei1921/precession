import type { QueryClient } from '@tanstack/react-query';
import { queryOptions } from '@tanstack/react-query';
import { bookGet, bookList, bookNoteList } from '@/bridge/library';

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

export async function refreshLibrary(client: QueryClient) {
  await client.invalidateQueries({ queryKey: ['library'] });
  await client.invalidateQueries({ queryKey: ['timeline'] });
}
