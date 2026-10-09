import { queryOptions } from '@tanstack/react-query';
import { taskList } from '@/bridge/task';

export const taskListQuery = queryOptions({
  queryKey: ['task', 'list'],
  queryFn: taskList
});
