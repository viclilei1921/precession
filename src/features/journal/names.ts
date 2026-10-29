import { useQuery } from '@tanstack/react-query';
import { memberListQuery } from '@/query/member/query';
import { placeListQuery } from '@/query/place/query';
import { tagListQuery } from '@/query/tag/query';

/** 手记上的成员、地点和标签 */
export function useJournalNames() {
  const membersQuery = useQuery(memberListQuery);
  const placesQuery = useQuery(placeListQuery);
  const tagsQuery = useQuery(tagListQuery);
  return {
    members: membersQuery.data ?? [],
    places: placesQuery.data ?? [],
    tags: tagsQuery.data ?? [],
    memberError: membersQuery.error,
    placeError: placesQuery.error,
    tagError: tagsQuery.error
  };
}
