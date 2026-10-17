import { useState } from 'react';
import type { Member, MemberInput } from '@/bridge/member';
import { Button } from '@/components/button';
import { ChipButton } from '@/components/button/chip';
import { DateField } from '@/components/date-field';
import styles from '@/components/record.module.css';
import { TextField } from '@/components/text-field';
import { ageLabel, fromDateInputValue, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';

type MemberPanelProps = {
  members: Member[];
  listError: string;
  isPending: boolean;
  selectedId?: string | null;
  onSelect?: (id: string) => void;
  onSave: (member: Member | null, input: MemberInput) => Promise<Member>;
  onDelete: (id: string) => Promise<void>;
};

export function MemberPanel({
  members,
  listError,
  isPending,
  selectedId,
  onSelect,
  onSave,
  onDelete
}: MemberPanelProps) {
  const [editing, setEditing] = useState<Member | 'new' | null>(null);

  return (
    <div className={styles.stack}>
      {listError ? <p className={styles.error}>{listError}</p> : null}
      <div className={styles.chips}>
        {members.map((member) => (
          <ChipButton
            key={member.id}
            selected={selectedId === member.id}
            onClick={() => {
              onSelect?.(member.id);
              if (!onSelect) {
                setEditing(member);
              }
            }}
          >
            {member.name}
            {member.birthday != null ? ` · ${ageLabel(member.birthday)}` : ''}
          </ChipButton>
        ))}
        <ChipButton onClick={() => setEditing('new')}>添加成员</ChipButton>
        {onSelect && selectedId ? (
          <ChipButton
            onClick={() => {
              const found = members.find((member) => member.id === selectedId);
              if (found) {
                setEditing(found);
              }
            }}
          >
            编辑成员
          </ChipButton>
        ) : null}
      </div>
      {editing !== null ? (
        <MemberForm
          key={editing === 'new' ? 'new' : editing.id}
          member={editing === 'new' ? null : editing}
          onClose={() => setEditing(null)}
          onSave={onSave}
          onDelete={onDelete}
          onSaved={(member) => {
            onSelect?.(member.id);
            setEditing(null);
          }}
        />
      ) : null}
      {members.length === 0 && !isPending ? (
        <p className={styles.empty}>还没有成员。先为自己或家人建一份档案。</p>
      ) : null}
    </div>
  );
}

function MemberForm({
  member,
  onClose,
  onSave,
  onDelete,
  onSaved
}: {
  member: Member | null;
  onClose: () => void;
  onSave: (member: Member | null, input: MemberInput) => Promise<Member>;
  onDelete: (id: string) => Promise<void>;
  onSaved: (member: Member) => void;
}) {
  const [name, setName] = useState(member?.name ?? '');
  const [relation, setRelation] = useState(member?.relation ?? '');
  const [gender, setGender] = useState(member?.gender ?? '');
  const [birthday, setBirthday] = useState(toDateInputValue(member?.birthday ?? null));
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState('');

  async function submit() {
    setPending(true);
    setError('');
    try {
      const saved = await onSave(member, {
        name,
        relation,
        gender,
        birthday: fromDateInputValue(birthday)
      });
      onSaved(saved);
    } catch (err) {
      setError(errorMessage(err));
      setPending(false);
    }
  }

  async function remove() {
    if (!member) {
      return;
    }
    setPending(true);
    setError('');
    try {
      await onDelete(member.id);
      onClose();
    } catch (err) {
      setError(errorMessage(err));
      setPending(false);
    }
  }

  return (
    <form
      className={styles.stack}
      onSubmit={(event) => {
        event.preventDefault();
        void submit();
      }}
    >
      <h3 className={styles.cardTitle}>{member ? '编辑成员' : '添加成员'}</h3>
      <TextField label="名字" value={name} required onChange={(event) => setName(event.target.value)} />
      <div className={styles.pair}>
        <TextField
          label="关系"
          value={relation}
          placeholder="自己、孩子、父母"
          onChange={(event) => setRelation(event.target.value)}
        />
        <TextField label="性别" value={gender} onChange={(event) => setGender(event.target.value)} />
      </div>
      <DateField label="生日" value={birthday} onChange={setBirthday} />
      {error ? <p className={styles.error}>{error}</p> : null}
      <div className={styles.actions}>
        {member ? (
          <Button
            variant="outlined"
            tone="destructive"
            disabled={pending}
            onClick={() => {
              if (!confirmDelete) {
                setConfirmDelete(true);
                return;
              }
              void remove();
            }}
          >
            {confirmDelete ? '确认删除' : '删除'}
          </Button>
        ) : null}
        <span className={styles.spacer} />
        <Button variant="outlined" disabled={pending} onClick={onClose}>
          取消
        </Button>
        <Button type="submit" className={styles.trailing} disabled={pending}>
          保存
        </Button>
      </div>
    </form>
  );
}
