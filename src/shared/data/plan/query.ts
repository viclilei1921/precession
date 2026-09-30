import type { QueryClient } from '@tanstack/react-query';
import { queryOptions } from '@tanstack/react-query';
import type { PlanQuery } from '@/bridge/plan';
import { planList } from '@/bridge/plan';
import { addLocalDays, addLocalMonths, startOfLocalDay, startOfLocalMonth } from '@/shared/lib/day';

export const planListQuery = queryOptions({
  queryKey: ['plan', 'list'],
  queryFn: () => planList()
});

/** 按时间或清单取计划。和全量列表共用 `plan` 前缀，写完会一起刷新。 */
export function planListQueryFor(query: PlanQuery) {
  return queryOptions({
    queryKey: ['plan', 'list', query],
    queryFn: () => planList(query)
  });
}

/** 某一本地日：当天 0 点到次日 0 点 */
export function planDayQuery(day: number, groupId?: string): PlanQuery {
  const from = startOfLocalDay(day);
  return groupId ? { from, to: addLocalDays(from, 1), groupId } : { from, to: addLocalDays(from, 1) };
}

/** 某一本地月：当月 1 日 0 点到下月 1 日 0 点 */
export function planMonthQuery(month: number, groupId?: string): PlanQuery {
  const from = startOfLocalMonth(month);
  return groupId ? { from, to: addLocalMonths(from, 1), groupId } : { from, to: addLocalMonths(from, 1) };
}

/** 计划和今天都读这份列表，写完要一起刷新时间轴 */
export async function refreshPlanViews(client: QueryClient) {
  await client.invalidateQueries({ queryKey: ['plan'] });
  await client.invalidateQueries({ queryKey: ['timeline'] });
}
