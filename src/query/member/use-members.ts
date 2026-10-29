import { useQuery, useQueryClient } from '@tanstack/react-query';
import type { Member, MemberInput } from '@/bridge/member';
import { memberCreate, memberDelete, memberUpdate } from '@/bridge/member';
import { refreshFiles } from '@/query/files/query';
import { memberListQuery, refreshMembers } from './query';

export function useMembers() {
  const queryClient = useQueryClient();
  const membersQuery = useQuery(memberListQuery);

  async function save(member: Member | null, input: MemberInput) {
    const saved = member ? await memberUpdate(member.id, input) : await memberCreate(input);
    await refreshMembers(queryClient);
    if (!member || member.avatar !== input.avatar) {
      await refreshFiles(queryClient);
    }
    return saved;
  }

  async function remove(id: string) {
    await memberDelete(id);
    await refreshMembers(queryClient);
    await refreshFiles(queryClient);
  }

  return {
    members: membersQuery.data ?? [],
    error: membersQuery.error,
    isPending: membersQuery.isPending,
    save,
    remove
  };
}
