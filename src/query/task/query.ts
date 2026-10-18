import type { QueryClient } from '@tanstack/react-query';
import { queryOptions } from '@tanstack/react-query';
import { taskList } from '@/bridge/task';

export const taskListQuery = queryOptions({
  queryKey: ['task', 'list'],
  queryFn: taskList
});

/** 入队、取消或进度事件之后刷新队列 */
export async function refreshTasks(client: QueryClient) {
  await client.invalidateQueries({ queryKey: ['task'] });
}
