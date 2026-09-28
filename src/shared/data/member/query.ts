import type { QueryClient } from '@tanstack/react-query';
import { queryOptions } from '@tanstack/react-query';
import { memberList } from '@/bridge/member';

export const memberListQuery = queryOptions({
  queryKey: ['member', 'list'],
  queryFn: memberList
});

export async function refreshMembers(client: QueryClient) {
  await client.invalidateQueries({ queryKey: ['member'] });
}
