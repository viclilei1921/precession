import { CameraIcon, FlagIcon, PlusIcon } from '@phosphor-icons/react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import type { GrowthEntry, GrowthKind } from '@/bridge/growth';
import { growthEntryCreate, growthEntryDelete, growthEntryUpdate } from '@/bridge/growth';
import { PATH } from '@/router/path';
import { growthListQuery, refreshGrowth } from '@/shared/data/growth/query';
import { MemberPanel } from '@/shared/data/member/panel';
import { memberListQuery } from '@/shared/data/member/query';
import { ageLabel, formatMonthDay, fromDateInputValue, startOfLocalDay, toDateInputValue } from '@/shared/lib/day';
import { errorMessage } from '@/shared/lib/error';
import { Dialog } from '@/shared/ui/dialog';
import styles from '@/shared/ui/record.module.css';

export function GrowthPage() {
  const navigate = useNavigate();
  const search = useSearch({ from: '/growth' });
  const membersQuery = useQuery(memberListQuery);
  const members = membersQuery.data ?? [];
  const [memberId, setMemberId] = useState<string | null>(null);
  const [kind, setKind] = useState<GrowthKind | 'all'>('all');
  const [editing, setEditing] = useState<GrowthEntry | 'new' | null>(null);
  const selected = members.find((item) => item.id === memberId)?.id ?? members[0]?.id ?? null;
  const listQuery = useQuery({
    ...growthListQuery(selected ?? undefined, kind === 'all' ? undefined : kind),
    enabled: selected != null
  });
  const entries = [...(listQuery.data ?? [])].sort((left, right) => right.occurredAt - left.occurredAt);
  const member = members.find((item) => item.id === selected) ?? null;

  useEffect(() => {
    if (search.kind) {
      setKind(search.kind);
    }
    if (search.create) {
      setEditing('new');
    }
    if (search.create || search.kind) {
      void navigate({ to: PATH.growth, search: {}, replace: true });
    }
  }, [navigate, search.create, search.kind]);

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>成长</h1>
        <p className={styles.hint}>按成员排成一条轨迹。里程碑记大事件，瞬间留一句和当时的年纪。</p>
        <div className={styles.segments} role="tablist" aria-label="成长类型">
          <button
            type="button"
            className={styles.segment}
            data-on={kind === 'all' ? 'true' : undefined}
            onClick={() => setKind('all')}
          >
            全部
          </button>
          <button
            type="button"
            className={styles.segment}
            data-on={kind === 'milestone' ? 'true' : undefined}
            onClick={() => setKind('milestone')}
          >
            <FlagIcon className={styles.icon} weight="regular" />
            里程碑
          </button>
          <button
            type="button"
            className={styles.segment}
            data-on={kind === 'moment' ? 'true' : undefined}
            onClick={() => setKind('moment')}
          >
            <CameraIcon className={styles.icon} weight="regular" />
            瞬间
          </button>
        </div>
        <button type="button" className={styles.primary} disabled={selected == null} onClick={() => setEditing('new')}>
          <PlusIcon className={styles.icon} weight="regular" />
          记一笔
        </button>
      </header>
      <MemberPanel selectedId={selected} onSelect={setMemberId} />
      {member ? (
        <p className={styles.note}>
          {member.relation ? `${member.relation} · ` : ''}
          {member.birthday != null ? `现在 ${ageLabel(member.birthday)}` : '还没有生日，年龄不会自动算'}
        </p>
      ) : null}
      {listQuery.error ? <p className={styles.error}>{errorMessage(listQuery.error)}</p> : null}
      <div className={styles.card}>
        {selected == null ? <p className={styles.empty}>先添加一位成员，再记里程碑和瞬间。</p> : null}
        {selected != null && listQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}
        {selected != null && !listQuery.isPending && entries.length === 0 ? (
          <p className={styles.empty}>这条轨迹还是空的。</p>
        ) : null}
        {entries.map((entry) => (
          <button key={entry.id} type="button" className={styles.row} onClick={() => setEditing(entry)}>
            <span className={styles.rowTitle}>
              {entry.kind === 'milestone' ? '里程碑' : '瞬间'} · {entry.title}
            </span>
            <span className={styles.meta}>
              {entry.highlight ? '高光 · ' : ''}
              {member?.birthday != null ? `${ageLabel(member.birthday, entry.occurredAt)} · ` : ''}
              {formatMonthDay(entry.occurredAt)}
            </span>
          </button>
        ))}
      </div>
      {editing !== null && selected ? (
        <GrowthDialog
          key={editing === 'new' ? 'new' : editing.id}
          memberId={selected}
          entry={editing === 'new' ? null : editing}
          kind={kind === 'all' ? 'milestone' : kind}
          onClose={() => setEditing(null)}
        />
      ) : null}
    </section>
  );
}

function GrowthDialog({
  memberId,
  entry,
  kind,
  onClose
}: {
  memberId: string;
  entry: GrowthEntry | null;
  kind: GrowthKind;
  onClose: () => void;
}) {
  const queryClient = useQueryClient();
  const [draftKind, setDraftKind] = useState<GrowthKind>(entry?.kind ?? kind);
  const [title, setTitle] = useState(entry?.title ?? '');
  const [body, setBody] = useState(entry?.body ?? '');
  const [occurredAt, setOccurredAt] = useState(toDateInputValue(entry?.occurredAt ?? startOfLocalDay()));
  const [highlight, setHighlight] = useState(entry?.highlight ?? false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const save = useMutation({
    mutationFn: () => {
      const input = {
        kind: draftKind,
        memberId,
        occurredAt: fromDateInputValue(occurredAt) ?? startOfLocalDay(),
        title,
        body,
        locked: entry?.locked ?? false,
        highlight,
        memberIds: entry?.memberIds ?? [],
        tagIds: entry?.tagIds ?? [],
        placeIds: entry?.placeIds ?? []
      };
      return entry ? growthEntryUpdate(entry.id, input) : growthEntryCreate(input);
    },
    onSuccess: async () => {
      await refreshGrowth(queryClient);
      onClose();
    }
  });
  const remove = useMutation({
    mutationFn: () => (entry ? growthEntryDelete(entry.id) : Promise.reject(new Error('记录不存在'))),
    onSuccess: async () => {
      await refreshGrowth(queryClient);
      onClose();
    }
  });
  const pending = save.isPending || remove.isPending;

  return (
    <Dialog title={entry ? '编辑成长' : '记下成长'} onClose={onClose}>
      <form
        className={styles.stack}
        onSubmit={(event) => {
          event.preventDefault();
          save.mutate();
        }}
      >
        <div className={styles.pair}>
          <label className={styles.field}>
            类型
            <select value={draftKind} onChange={(event) => setDraftKind(event.target.value as GrowthKind)}>
              <option value="milestone">里程碑</option>
              <option value="moment">瞬间</option>
            </select>
          </label>
          <label className={styles.field}>
            发生时间
            <input type="date" value={occurredAt} onChange={(event) => setOccurredAt(event.target.value)} />
          </label>
        </div>
        <label className={styles.field}>
          标题
          <input value={title} required onChange={(event) => setTitle(event.target.value)} />
        </label>
        <label className={styles.field}>
          说明
          <textarea value={body} onChange={(event) => setBody(event.target.value)} />
        </label>
        <label className={styles.check}>
          <input type="checkbox" checked={highlight} onChange={(event) => setHighlight(event.target.checked)} />
          标为高光，进入年度之书
        </label>
        {save.error ? <p className={styles.error}>{errorMessage(save.error)}</p> : null}
        {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
        <div className={styles.actions}>
          {entry ? (
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
