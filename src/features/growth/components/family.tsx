import { CaretLeftIcon, PlusIcon } from '@phosphor-icons/react';
import { useRef, useState } from 'react';
import { pickFiles } from '@/bridge/files';
import type { Member, MemberInput } from '@/bridge/member';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { DateField } from '@/components/date-field';
import { discardImages, importImagePath, looseImageOwner } from '@/components/markdown';
import { Modal } from '@/components/modal';
import { Scroll } from '@/components/scroll';
import { TextField } from '@/components/text-field';
import { useMembers } from '@/query/member/use-members';
import { useAppStore } from '@/store/app';
import { ageLabel, fromDateInputValue, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import styles from '../page.module.css';
import { MemberFace } from './face';

const imageExts = ['png', 'jpg', 'jpeg', 'gif', 'webp', 'avif'];

type FamilyViewProps = {
  onBack: () => void;
  onRemoved: (id: string) => void;
};

/** 家人列表。添加和修改用弹层，返回时回到进来之前的那一屏。 */
export function FamilyView({ onBack, onRemoved }: FamilyViewProps) {
  const membersState = useMembers();
  const [editing, setEditing] = useState<Member | 'new' | null>(null);

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <IconButton label="返回" onClick={onBack}>
          <CaretLeftIcon weight="regular" />
        </IconButton>
        <h1 className={styles.title}>家人</h1>
        <span className={styles.spacer} />
        <Button onClick={() => setEditing('new')}>
          <PlusIcon weight="regular" />
          添加
        </Button>
      </header>
      <Scroll className={styles.scroll}>
        <div className={styles.body}>
          {membersState.error ? <p className={styles.error}>{errorMessage(membersState.error)}</p> : null}
          {membersState.isPending ? <p className={styles.note}>正在读取…</p> : null}
          {!membersState.isPending && membersState.members.length === 0 ? (
            <p className={styles.empty}>先添加一位家人。</p>
          ) : null}
          {membersState.members.length > 0 ? (
            <div className={styles.people}>
              {membersState.members.map((member) => {
                const age = member.birthday != null ? ageLabel(member.birthday) : '';
                const meta = [member.relation, age || (member.birthday == null ? '还没有生日' : '')]
                  .filter(Boolean)
                  .join(' · ');
                return (
                  <button key={member.id} type="button" className={styles.person} onClick={() => setEditing(member)}>
                    <MemberFace
                      name={member.name}
                      imageId={member.avatar}
                      owner={{ owner: 'member', ownerId: member.id }}
                    />
                    <span className={styles.personCopy}>
                      <span className={styles.personName}>{member.name}</span>
                      {meta ? <span className={styles.personMeta}>{meta}</span> : null}
                    </span>
                  </button>
                );
              })}
            </div>
          ) : null}
        </div>
      </Scroll>
      {editing !== null ? (
        <MemberForm
          key={editing === 'new' ? 'new' : editing.id}
          member={editing === 'new' ? null : editing}
          onClose={() => setEditing(null)}
          onSave={membersState.save}
          onDelete={async (id) => {
            await membersState.remove(id);
            onRemoved(id);
          }}
          onSaved={() => setEditing(null)}
        />
      ) : null}
    </section>
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
  onSaved: () => void;
}) {
  const [name, setName] = useState(member?.name ?? '');
  const [relation, setRelation] = useState(member?.relation ?? '');
  const [gender, setGender] = useState(member?.gender ?? '');
  const [birthday, setBirthday] = useState(toDateInputValue(member?.birthday ?? null));
  const [avatar, setAvatar] = useState(member?.avatar ?? '');
  const [avatarLoose, setAvatarLoose] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [pending, setPending] = useState(false);
  const [importing, setImporting] = useState(false);
  const [error, setError] = useState('');
  const session = useRef(new Set<string>());
  const faceOwner = avatarLoose || member == null ? looseImageOwner : { owner: 'member' as const, ownerId: member.id };
  const busy = pending || importing;

  async function dropSession(keep: string) {
    const drop = [...session.current].filter((id) => id !== keep);
    session.current.clear();
    await discardImages(drop);
  }

  function close() {
    if (busy) {
      return;
    }
    void dropSession('');
    onClose();
  }

  async function pickAvatar() {
    const paths = await pickFiles({ multiple: false, extensions: imageExts });
    const path = paths[0];
    if (!path) {
      return;
    }
    setImporting(true);
    setError('');
    try {
      const id = await importImagePath(path, looseImageOwner, !useAppStore.getState().isMobile());
      session.current.add(id);
      setAvatar(id);
      setAvatarLoose(true);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setImporting(false);
    }
  }

  async function submit() {
    setPending(true);
    setError('');
    try {
      await onSave(member, {
        name,
        relation,
        gender,
        birthday: fromDateInputValue(birthday),
        avatar
      });
      await dropSession(avatar);
      onSaved();
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
      await dropSession('');
      onClose();
    } catch (err) {
      setError(errorMessage(err));
      setPending(false);
    }
  }

  return (
    <Modal title={member ? '编辑家人' : '添加家人'} onClose={close}>
      <form
        className={styles.stack}
        onSubmit={(event) => {
          event.preventDefault();
          void submit();
        }}
      >
        <div className={styles.avatarRow}>
          <button
            type="button"
            className={styles.avatarPick}
            aria-label={avatar ? '更换头像' : '上传头像'}
            disabled={busy}
            onClick={() => void pickAvatar()}
          >
            <MemberFace name={name || '人'} imageId={avatar} owner={faceOwner} size="lg" />
          </button>
          <div className={styles.avatarCopy}>
            <p className={styles.hint}>
              {importing ? '正在导入图片…' : avatar ? '点头像可以换一张。' : '点头像，从本机选一张图片。'}
            </p>
            {avatar ? (
              <Button variant="text" disabled={busy} onClick={() => setAvatar('')}>
                移除头像
              </Button>
            ) : null}
          </div>
        </div>
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
        {confirmDelete ? <p className={styles.hint}>再点一次就会删掉这位家人。</p> : null}
        <div className={styles.actions}>
          {member ? (
            <Button
              variant="outlined"
              tone="destructive"
              disabled={busy}
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
          <Button variant="outlined" disabled={busy} onClick={close}>
            取消
          </Button>
          <Button type="submit" disabled={busy}>
            保存
          </Button>
        </div>
      </form>
    </Modal>
  );
}
