import { queryOptions } from '@tanstack/react-query';
import { memberList } from '@/bridge/member';

export const memberListQuery = queryOptions({
  queryKey: ['member', 'list'],
  queryFn: memberList
});
