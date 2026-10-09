import { useState } from 'react';
import type { Member, MemberInput } from '@/bridge/member';
import { ageLabel, fromDateInputValue, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import { Dialog } from './dialog';
import styles from './record.module.css';

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
      {members.length === 0 && !isPending ? (
        <p className={styles.empty}>还没有成员。先为自己或家人建一份档案。</p>
      ) : null}
      {editing !== null ? (
        <MemberDialog
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
    </div>
  );
}

function MemberDialog({
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
    <Dialog title={member ? '编辑成员' : '添加成员'} onClose={onClose}>
      <form
        className={styles.stack}
        onSubmit={(event) => {
          event.preventDefault();
          void submit();
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
        {error ? <p className={styles.error}>{error}</p> : null}
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
                void remove();
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
