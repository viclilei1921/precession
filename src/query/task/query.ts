import type { QueryClient } from '@tanstack/react-query';
import { queryOptions } from '@tanstack/react-query';
import type { Task } from '@/bridge/task';
import { taskList } from '@/bridge/task';

export const taskListQuery = queryOptions({
  queryKey: ['task', 'list'],
  queryFn: taskList
});

/** 入队、取消或整队变化之后重新读取队列。 */
export async function refreshTasks(client: QueryClient) {
  await client.invalidateQueries({ queryKey: ['task'] });
}

/** 把单条任务的进度写进当前列表，避免每拍都重新请求。 */
export function applyTaskUpdate(client: QueryClient, task: Task) {
  client.setQueryData<Task[]>(taskListQuery.queryKey, (current) => {
    if (!current) {
      return [task];
    }
    const index = current.findIndex((item) => item.id === task.id);
    if (index < 0) {
      return [task, ...current];
    }
    const next = current.slice();
    next[index] = task;
    return next;
  });
}
