import { CameraIcon, FlagIcon, PlusIcon, SquaresFourIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import type { GrowthEntry, GrowthKind } from '@/bridge/growth';
import { MemberPanel } from '@/components/member-panel';
import styles from '@/components/record.module.css';
import { usePageSearch } from '@/hooks/search';
import { growthListQuery } from '@/query/growth/query';
import { useMembers } from '@/query/member/use-members';
import { PATH } from '@/router/path';
import { ageLabel, formatMonthDay } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import { matchesQuery } from '@/utils/search';
import { GrowthForm } from './components/growth-form';

export function GrowthPage() {
  const navigate = useNavigate();
  const search = useSearch({ from: '/growth' });
  const membersState = useMembers();
  const members = membersState.members;
  const query = usePageSearch('搜索成长');
  const [memberId, setMemberId] = useState<string | null>(null);
  const [kind, setKind] = useState<GrowthKind | 'all'>('all');
  const [editing, setEditing] = useState<GrowthEntry | 'new' | null>(null);
  const selected = members.find((item) => item.id === memberId)?.id ?? members[0]?.id ?? null;
  const listQuery = useQuery({
    ...growthListQuery(selected ?? undefined, kind === 'all' ? undefined : kind),
    enabled: selected != null
  });
  const entries = [...(listQuery.data ?? [])]
    .filter((entry) => matchesQuery(query, entry.title, entry.body))
    .sort((left, right) => right.occurredAt - left.occurredAt);
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
            <SquaresFourIcon className={styles.icon} weight="regular" />
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
      <MemberPanel
        members={members}
        listError={membersState.error ? errorMessage(membersState.error) : ''}
        isPending={membersState.isPending}
        selectedId={selected}
        onSelect={setMemberId}
        onSave={membersState.save}
        onDelete={membersState.remove}
      />
      {member ? (
        <p className={styles.note}>
          {member.relation ? `${member.relation} · ` : ''}
          {member.birthday != null ? `现在 ${ageLabel(member.birthday)}` : '还没有生日，年龄不会自动算'}
        </p>
      ) : null}
      {listQuery.error ? <p className={styles.error}>{errorMessage(listQuery.error)}</p> : null}
      {editing !== null && selected ? (
        <div className={styles.card}>
          <h2 className={styles.cardTitle}>{editing === 'new' ? '记下成长' : '编辑成长'}</h2>
          <GrowthForm
            key={editing === 'new' ? 'new' : editing.id}
            memberId={selected}
            entry={editing === 'new' ? null : editing}
            kind={kind === 'all' ? 'milestone' : kind}
            onClose={() => setEditing(null)}
          />
        </div>
      ) : null}
      <div className={styles.card}>
        {selected == null ? <p className={styles.empty}>先添加一位成员，再记里程碑和瞬间。</p> : null}
        {selected != null && listQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}
        {selected != null && !listQuery.isPending && entries.length === 0 ? (
          <p className={styles.empty}>{query.trim() ? '没有匹配的记录。' : '这条轨迹还是空的。'}</p>
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
    </section>
  );
}
