import { queryOptions } from '@tanstack/react-query';
import type { TimelineKind } from '@/bridge/timeline';
import { timelineList } from '@/bridge/timeline';

export function timelineListQuery(from?: number, to?: number, kinds?: TimelineKind[]) {
  return queryOptions({
    queryKey: ['timeline', 'list', from ?? null, to ?? null, kinds ?? null],
    queryFn: () => timelineList(from, to, kinds)
  });
}
