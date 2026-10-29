import type { QueryClient } from '@tanstack/react-query';
import { queryOptions } from '@tanstack/react-query';
import { planGroupList, planList } from '@/bridge/plan';

export const planListQuery = queryOptions({
  queryKey: ['plan', 'list'],
  queryFn: () => planList()
});

export const planGroupListQuery = queryOptions({
  queryKey: ['plan', 'groups'],
  queryFn: () => planGroupList()
});

/** 计划和今天都读这份列表，写完要一起刷新时间轴 */
export async function refreshPlanViews(client: QueryClient) {
  await client.invalidateQueries({ queryKey: ['plan'] });
  await client.invalidateQueries({ queryKey: ['timeline'] });
}
