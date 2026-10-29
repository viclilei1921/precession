import type { QueryClient } from '@tanstack/react-query';
import { queryOptions } from '@tanstack/react-query';
import type { OwnerKind } from '@/bridge/upload';
import { mediaInfo, uploadCatalog } from '@/bridge/upload';
import { refreshGrowth } from '@/query/growth/query';
import { refreshJournal } from '@/query/journal/query';
import { refreshLibrary } from '@/query/library/query';
import { refreshMembers } from '@/query/member/query';
import { refreshPlanViews } from '@/query/plan/query';

export const filesCatalogQuery = queryOptions({
  queryKey: ['files', 'catalog'],
  queryFn: uploadCatalog
});

export const mediaInfoQuery = queryOptions({
  queryKey: ['files', 'media'],
  queryFn: mediaInfo,
  staleTime: Number.POSITIVE_INFINITY
});

export async function refreshFiles(client: QueryClient) {
  await client.invalidateQueries({ queryKey: ['files', 'catalog'] });
}

/** 删掉文件后，刷新它所属记录还在用的列表 */
export async function refreshFileOwner(client: QueryClient, owner: OwnerKind) {
  if (owner === 'loose') {
    return;
  }
  if (owner === 'journal_entry') {
    await refreshJournal(client);
    return;
  }
  if (owner === 'book' || owner === 'book_note') {
    await refreshLibrary(client);
    return;
  }
  if (owner === 'plan') {
    await refreshPlanViews(client);
    return;
  }
  if (owner === 'member') {
    await refreshMembers(client);
    return;
  }
  await refreshGrowth(client);
}
