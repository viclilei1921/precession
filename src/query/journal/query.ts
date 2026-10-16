import type { QueryClient } from '@tanstack/react-query';
import { queryOptions } from '@tanstack/react-query';
import type { JournalKind } from '@/bridge/journal';
import { journalCitationList, journalEntryGet, journalEntryList, journalLinkList } from '@/bridge/journal';

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

export function journalLinkListQuery(entryId: string) {
  return queryOptions({
    queryKey: ['journal', 'links', entryId],
    queryFn: () => journalLinkList(entryId)
  });
}

export function journalCitationListQuery(entryId: string) {
  return queryOptions({
    queryKey: ['journal', 'citations', entryId],
    queryFn: () => journalCitationList(entryId)
  });
}

export async function refreshJournal(client: QueryClient) {
  await client.invalidateQueries({ queryKey: ['journal'] });
  await client.invalidateQueries({ queryKey: ['timeline'] });
}

export async function refreshJournalCitations(client: QueryClient, entryId: string) {
  await client.invalidateQueries({ queryKey: ['journal', 'citations', entryId] });
}
