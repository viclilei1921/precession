import type { QueryClient } from '@tanstack/react-query';
import { queryOptions } from '@tanstack/react-query';
import type { JournalKind } from '@/bridge/journal';
import { journalEntryGet, journalEntryList } from '@/bridge/journal';

export function journalListQuery(kind?: JournalKind) {
  return queryOptions({
    queryKey: ['journal', 'list', kind ?? null],
    queryFn: () => journalEntryList(kind)
  });
}

export function journalEntryQuery(id: string) {
  return queryOptions({
    queryKey: ['journal', 'entry', id],
    queryFn: () => journalEntryGet(id)
  });
}

export async function refreshJournal(client: QueryClient) {
  await client.invalidateQueries({ queryKey: ['journal'] });
  await client.invalidateQueries({ queryKey: ['timeline'] });
}
