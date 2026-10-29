import { queryOptions } from '@tanstack/react-query';
import { archiveInfo } from '@/bridge/archive';

export const archiveInfoQuery = queryOptions({
  queryKey: ['archive', 'info'],
  queryFn: archiveInfo
});
