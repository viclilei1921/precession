import type { QueryClient } from '@tanstack/react-query';
import { queryOptions } from '@tanstack/react-query';
import { tagList } from '@/bridge/tag';

export const tagListQuery = queryOptions({
  queryKey: ['tag', 'list'],
  queryFn: tagList
});

export async function refreshTags(client: QueryClient) {
  await client.invalidateQueries({ queryKey: ['tag'] });
}
