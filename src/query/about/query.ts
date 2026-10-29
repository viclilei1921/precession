import { queryOptions } from '@tanstack/react-query';
import { appVersions } from '@/bridge/about';

export const appVersionsQuery = queryOptions({
  queryKey: ['about', 'versions'],
  queryFn: appVersions,
  staleTime: Number.POSITIVE_INFINITY
});
