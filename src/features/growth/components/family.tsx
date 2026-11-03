import { CaretLeftIcon, PlusIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { pickFiles } from '@/bridge/files';
import type { GrowthMeasure } from '@/bridge/growth';
import type { Member, MemberInput } from '@/bridge/member';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { DateField } from '@/components/date-field';
import { discardImages, importImagePath, looseImageOwner } from '@/components/markdown';
import { Modal, ModalCloseRef } from '@/components/modal';
import { Scroll } from '@/components/scroll';
import { TextField } from '@/components/text-field';
import { journalDate } from '@/features/journal/text';
import { useBackLayer } from '@/layout/mobile/back';
import { growthMeasureListQuery } from '@/query/growth/query';
import { useMembers } from '@/query/member/use-members';
import { useAppStore } from '@/store/app';
import { ageLabel, fromDateInputValue, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import styles from '../page.module.css';
import { cmText, kgText } from '../text';
import { MemberFace } from './face';
import { MeasureForm } from './measure-form';

const imageExts = ['png', 'jpg', 'jpeg', 'gif', 'webp', 'avif'];

type FamilyViewProps = {
  onBack: () => void;
  onRemoved: (id: string) => void;
  /** 传入后，添加和编辑打开独立页面，不再用弹层 */
  onEdit?: (target: Member | 'new') => void;
};

/** 家人列表。桌面在弹层里改档案；移动端改走独立页面。 */
export function FamilyView({ onBack, onRemoved, onEdit }: FamilyViewProps) {
  const { t } = useTranslation('growth');
  const membersState = useMembers();
  const [editing, setEditing] = useState<Member | 'new' | null>(null);

  function openEditor(target: Member | 'new') {
    if (onEdit) {
      onEdit(target);
      return;
    }
    setEditing(target);
  }

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <IconButton label={t('page.back')} onClick={onBack}>
          <CaretLeftIcon weight="regular" />
        </IconButton>
        <h1 className={styles.title}>{t('page.family')}</h1>
        <span className={styles.spacer} />
        <Button onClick={() => openEditor('new')}>
          <PlusIcon weight="regular" />
          {t('page.add')}
        </Button>
      </header>
      <Scroll className={styles.scroll}>
        <div className={styles.body}>
          {membersState.error ? <p className={styles.error}>{errorMessage(membersState.error)}</p> : null}
          {membersState.isPending ? <p className={styles.note}>{t('page.loading')}</p> : null}
          {!membersState.isPending && membersState.members.length === 0 ? (
            <p className={styles.empty}>{t('page.addFamilyFirst')}</p>
          ) : null}
          {membersState.members.length > 0 ? (
            <div className={styles.people}>
              {membersState.members.map((member) => {
                const age = member.birthday != null ? ageLabel(member.birthday) : '';
                const meta = [member.relation, age || (member.birthday == null ? t('page.noBirthday') : '')]
                  .filter(Boolean)
                  .join(' · ');
                return (
                  <button key={member.id} type="button" className={styles.person} onClick={() => openEditor(member)}>
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

/** 移动端编辑家人。整页，不是底部抽屉。已有家人时，身高体重也在这一页。 */
export function MemberEditPage({
  member,
  onClose,
  onRemoved,
  onCreated
}: {
  member: Member | null;
  onClose: () => void;
  onRemoved: (id: string) => void;
  onCreated?: (member: Member) => void;
}) {
  const membersState = useMembers();
  return (
    <MemberForm
      asPage
      member={member}
      onClose={onClose}
      onSave={membersState.save}
      onDelete={async (id) => {
        await membersState.remove(id);
        onRemoved(id);
      }}
      onSaved={onClose}
      onCreated={onCreated}
    />
  );
}

function MemberMeasures({ memberId }: { memberId: string }) {
  const { t } = useTranslation('growth');
  const query = useQuery(growthMeasureListQuery(memberId));
  const measures = query.data ?? [];
  const latest = measures[0] ?? null;
  const [editing, setEditing] = useState<GrowthMeasure | 'new' | null>(null);

  return (
    <section className={styles.measure}>
      <div className={styles.measureTop}>
        <h2>{t('measure.heightWeight')}</h2>
        <Button variant="text" onClick={() => setEditing('new')}>
          {t('page.recordMeasure')}
        </Button>
      </div>
      {query.error ? <p className={styles.error}>{errorMessage(query.error)}</p> : null}
      {query.isPending ? <p className={styles.note}>{t('page.loading')}</p> : null}
      {latest ? (
        <p className={styles.measureNow}>
          <strong>
            {cmText(latest.heightMm)} <span>{t('page.centimeter')}</span>
          </strong>
          <strong>
            {kgText(latest.weightG)} <span>{t('page.kilogram')}</span>
          </strong>
        </p>
      ) : null}
      {!query.isPending && measures.length === 0 ? <p className={styles.note}>{t('page.measureHint')}</p> : null}
      {measures.length > 0 ? (
        <ul className={styles.measureList}>
          {measures.map((item) => (
            <li key={item.id}>
              <button type="button" className={styles.measureRow} onClick={() => setEditing(item)}>
                <span>{journalDate(item.occurredAt).full}</span>
                <span>
                  {cmText(item.heightMm)} {t('page.centimeter')}
                </span>
                <span>
                  {kgText(item.weightG)} {t('page.kilogram')}
                </span>
              </button>
            </li>
          ))}
        </ul>
      ) : null}
      {editing !== null ? (
        <MeasureForm
          key={editing === 'new' ? 'new' : editing.id}
          memberId={memberId}
          measure={editing === 'new' ? null : editing}
          onClose={() => setEditing(null)}
        />
      ) : null}
    </section>
  );
}

function MemberForm({
  member,
  asPage = false,
  onClose,
  onSave,
  onDelete,
  onSaved,
  onCreated
}: {
  member: Member | null;
  asPage?: boolean;
  onClose: () => void;
  onSave: (member: Member | null, input: MemberInput) => Promise<Member>;
  onDelete: (id: string) => Promise<void>;
  onSaved: () => void;
  onCreated?: (member: Member) => void;
}) {
  const { t } = useTranslation('growth');
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
  const closeRef = useRef(onClose);
  closeRef.current = onClose;
  const faceOwner = avatarLoose || member == null ? looseImageOwner : { owner: 'member' as const, ownerId: member.id };
  const busy = pending || importing;
  useBackLayer(!asPage, onClose);

  async function dropSession(keep: string) {
    const drop = [...session.current].filter((id) => id !== keep);
    session.current.clear();
    await discardImages(drop);
  }

  function finish() {
    void dropSession('');
    onClose();
    onSaved();
  }

  function close() {
    closeRef.current();
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
      const saved = await onSave(member, {
        name,
        relation,
        gender,
        birthday: fromDateInputValue(birthday),
        avatar
      });
      await dropSession(avatar);
      if (asPage && member == null && onCreated) {
        onCreated(saved);
        setPending(false);
        return;
      }
      closeRef.current();
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
      closeRef.current();
    } catch (err) {
      setError(errorMessage(err));
      setPending(false);
    }
  }

  const fields = (
    <>
      <div className={styles.avatarRow}>
        <button
          type="button"
          className={styles.avatarPick}
          aria-label={avatar ? t('form.changeAvatar') : t('form.uploadAvatar')}
          disabled={busy}
          onClick={() => void pickAvatar()}
        >
          <MemberFace name={name || t('page.personFallback')} imageId={avatar} owner={faceOwner} size="lg" />
        </button>
        <div className={styles.avatarCopy}>
          <p className={styles.hint}>
            {importing ? t('form.importingImage') : avatar ? t('form.changeAvatarHint') : t('form.pickAvatarHint')}
          </p>
          {avatar ? (
            <Button variant="text" disabled={busy} onClick={() => setAvatar('')}>
              {t('form.removeAvatar')}
            </Button>
          ) : null}
        </div>
      </div>
      <TextField label={t('form.name')} value={name} required onChange={(event) => setName(event.target.value)} />
      <div className={styles.pair}>
        <TextField
          label={t('form.relation')}
          value={relation}
          placeholder={t('form.relationPlaceholder')}
          onChange={(event) => setRelation(event.target.value)}
        />
        <TextField label={t('form.gender')} value={gender} onChange={(event) => setGender(event.target.value)} />
      </div>
      <DateField label={t('form.birthday')} value={birthday} onChange={setBirthday} />
      {error ? <p className={styles.error}>{error}</p> : null}
    </>
  );

  if (asPage) {
    return (
      <section className={styles.page}>
        <header className={styles.head}>
          <IconButton label={t('page.back')} disabled={busy} onClick={onClose}>
            <CaretLeftIcon weight="regular" />
          </IconButton>
          <h1 className={styles.title}>{member ? t('form.editMember') : t('page.addFamily')}</h1>
          <span className={styles.spacer} />
          <Button disabled={busy} onClick={() => void submit()}>
            {t('common:button.save')}
          </Button>
        </header>
        <Scroll className={styles.scroll}>
          <div className={styles.body}>
            <form
              className={styles.stack}
              onSubmit={(event) => {
                event.preventDefault();
                void submit();
              }}
            >
              {fields}
            </form>
            {member ? <MemberMeasures memberId={member.id} /> : null}
            {member ? (
              <div className={styles.stack}>
                {confirmDelete ? <p className={styles.hint}>{t('form.confirmDeleteMember')}</p> : null}
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
                  {confirmDelete ? t('form.confirmDelete') : t('common:button.delete')}
                </Button>
              </div>
            ) : null}
          </div>
        </Scroll>
      </section>
    );
  }

  return (
    <Modal title={member ? t('form.editMember') : t('page.addFamily')} onClose={finish}>
      <ModalCloseRef target={closeRef} />
      <form
        className={styles.stack}
        onSubmit={(event) => {
          event.preventDefault();
          void submit();
        }}
      >
        {fields}
        {confirmDelete ? <p className={styles.hint}>{t('form.confirmDeleteMember')}</p> : null}
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
              {confirmDelete ? t('form.confirmDelete') : t('common:button.delete')}
            </Button>
          ) : null}
          <span className={styles.spacer} />
          <Button variant="outlined" disabled={busy} onClick={close}>
            {t('common:button.cancel')}
          </Button>
          <Button type="submit" disabled={busy}>
            {t('common:button.save')}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
