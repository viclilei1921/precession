import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import type { Member } from '@/bridge/member';
import { memberCreate, memberDelete, memberUpdate } from '@/bridge/member';
import { ageLabel, fromDateInputValue, toDateInputValue } from '@/shared/lib/day';
import { errorMessage } from '@/shared/lib/error';
import { Dialog } from '@/shared/ui/dialog';
import styles from '@/shared/ui/record.module.css';
import { memberListQuery, refreshMembers } from './query';

type MemberPanelProps = {
  selectedId?: string | null;
  onSelect?: (id: string) => void;
};

export function MemberPanel({ selectedId, onSelect }: MemberPanelProps) {
  const queryClient = useQueryClient();
  const membersQuery = useQuery(memberListQuery);
  const members = membersQuery.data ?? [];
  const [editing, setEditing] = useState<Member | 'new' | null>(null);

  return (
    <div className={styles.stack}>
      {membersQuery.error ? <p className={styles.error}>{errorMessage(membersQuery.error)}</p> : null}
      <div className={styles.chips}>
        {members.map((member) => (
          <button
            key={member.id}
            type="button"
            className={styles.chip}
            data-on={selectedId === member.id ? 'true' : undefined}
            onClick={() => {
              onSelect?.(member.id);
              if (!onSelect) {
                setEditing(member);
              }
            }}
          >
            {member.name}
            {member.birthday != null ? ` · ${ageLabel(member.birthday)}` : ''}
          </button>
        ))}
        <button type="button" className={styles.chip} onClick={() => setEditing('new')}>
          添加成员
        </button>
        {onSelect && selectedId ? (
          <button
            type="button"
            className={styles.chip}
            onClick={() => {
              const found = members.find((member) => member.id === selectedId);
              if (found) {
                setEditing(found);
              }
            }}
          >
            编辑成员
          </button>
        ) : null}
      </div>
      {members.length === 0 && !membersQuery.isPending ? (
        <p className={styles.empty}>还没有成员。先为自己或家人建一份档案。</p>
      ) : null}
      {editing !== null ? (
        <MemberDialog
          member={editing === 'new' ? null : editing}
          onClose={() => setEditing(null)}
          onSaved={async (member) => {
            await refreshMembers(queryClient);
            onSelect?.(member.id);
            setEditing(null);
          }}
        />
      ) : null}
    </div>
  );
}

function MemberDialog({
  member,
  onClose,
  onSaved
}: {
  member: Member | null;
  onClose: () => void;
  onSaved: (member: Member) => Promise<void>;
}) {
  const queryClient = useQueryClient();
  const [name, setName] = useState(member?.name ?? '');
  const [relation, setRelation] = useState(member?.relation ?? '');
  const [gender, setGender] = useState(member?.gender ?? '');
  const [birthday, setBirthday] = useState(toDateInputValue(member?.birthday ?? null));
  const [confirmDelete, setConfirmDelete] = useState(false);
  const save = useMutation({
    mutationFn: () => {
      const input = {
        name,
        relation,
        gender,
        birthday: fromDateInputValue(birthday)
      };
      return member ? memberUpdate(member.id, input) : memberCreate(input);
    },
    onSuccess: (saved) => onSaved(saved)
  });
  const remove = useMutation({
    mutationFn: () => {
      if (!member) {
        return Promise.reject(new Error('成员不存在'));
      }
      return memberDelete(member.id);
    },
    onSuccess: async () => {
      await refreshMembers(queryClient);
      onClose();
    }
  });
  const pending = save.isPending || remove.isPending;

  return (
    <Dialog title={member ? '编辑成员' : '添加成员'} onClose={onClose}>
      <form
        className={styles.stack}
        onSubmit={(event) => {
          event.preventDefault();
          save.mutate();
        }}
      >
        <label className={styles.field}>
          名字
          <input value={name} required onChange={(event) => setName(event.target.value)} />
        </label>
        <div className={styles.pair}>
          <label className={styles.field}>
            关系
            <input
              value={relation}
              placeholder="自己、孩子、父母"
              onChange={(event) => setRelation(event.target.value)}
            />
          </label>
          <label className={styles.field}>
            性别
            <input value={gender} onChange={(event) => setGender(event.target.value)} />
          </label>
        </div>
        <label className={styles.field}>
          生日
          <input type="date" value={birthday} onChange={(event) => setBirthday(event.target.value)} />
        </label>
        {save.error ? <p className={styles.error}>{errorMessage(save.error)}</p> : null}
        {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
        <div className={styles.actions}>
          {member ? (
            <button
              type="button"
              className={styles.danger}
              disabled={pending}
              onClick={() => {
                if (!confirmDelete) {
                  setConfirmDelete(true);
                  return;
                }
                remove.mutate();
              }}
            >
              {confirmDelete ? '确认删除' : '删除'}
            </button>
          ) : null}
          <span className={styles.spacer} />
          <button type="button" className={styles.ghost} disabled={pending} onClick={onClose}>
            取消
          </button>
          <button type="submit" className={styles.primary} disabled={pending}>
            保存
          </button>
        </div>
      </form>
    </Dialog>
  );
}
