import type { QueryClient } from '@tanstack/react-query';
import { queryOptions } from '@tanstack/react-query';
import type { GrowthKind } from '@/bridge/growth';
import { growthEntryGet, growthEntryList, growthMeasureList } from '@/bridge/growth';

export function growthListQuery(memberId?: string, kind?: GrowthKind) {
  return queryOptions({
    queryKey: ['growth', 'list', memberId ?? null, kind ?? null],
    queryFn: () => growthEntryList(memberId, kind)
  });
}

export function growthEntryQuery(id: string) {
  return queryOptions({
    queryKey: ['growth', 'entry', id],
    queryFn: () => growthEntryGet(id)
  });
}

export function growthMeasureListQuery(memberId: string) {
  return queryOptions({
    queryKey: ['growth', 'measure', memberId],
    queryFn: () => growthMeasureList(memberId)
  });
}

export async function refreshGrowth(client: QueryClient) {
  await client.invalidateQueries({ queryKey: ['growth'] });
  await client.invalidateQueries({ queryKey: ['timeline'] });
}
