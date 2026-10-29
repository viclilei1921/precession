import type { QueryClient } from '@tanstack/react-query';
import { queryOptions } from '@tanstack/react-query';
import { placeList } from '@/bridge/place';

export const placeListQuery = queryOptions({
  queryKey: ['place', 'list'],
  queryFn: placeList
});

export async function refreshPlaces(client: QueryClient) {
  await client.invalidateQueries({ queryKey: ['place'] });
}
