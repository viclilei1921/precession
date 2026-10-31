import { PlusIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import type { GrowthKind } from '@/bridge/growth';
import { IconButton } from '@/components/button/icon';
import { SegmentButton } from '@/components/button/segment';
import { MemberFace } from '@/features/growth/components/face';
import { FamilyView } from '@/features/growth/components/family';
import { MeasureForm } from '@/features/growth/components/measure-form';
import { MobileGrowthEditor } from '@/features/growth/mobile/editor';
import { cmText, kgText } from '@/features/growth/text';
import { journalExcerpt } from '@/features/journal/text';
import { AppBar } from '@/layout/mobile/app-bar';
import { growthListQuery, growthMeasureListQuery } from '@/query/growth/query';
import { useMembers } from '@/query/member/use-members';
import { PATH } from '@/router/path';
import { ageLabel, formatMonthDay } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import { matchesQuery } from '@/utils/search';
import styles from './page.module.css';

/** 移动端成长：家人、最近一次测量、按年排的记录。 */
export function MobileGrowthPage() {
  const navigate = useNavigate();
  const search = useSearch({ from: '/growth' });
  const membersState = useMembers();
  const members = membersState.members;
  const member = members.find((item) => item.id === search.member) ?? members[0] ?? null;
  const [kind, setKind] = useState<GrowthKind | 'all'>('all');
  const [query, setQuery] = useState('');
  const listQuery = useQuery(growthListQuery(member?.id));
  const measuresQuery = useQuery({
    ...growthMeasureListQuery(member?.id ?? ''),
    enabled: member != null
  });
  const latest = measuresQuery.data?.[0] ?? null;
  const entries = [...(listQuery.data ?? [])]
    .filter(
      (entry) =>
        (kind === 'all' || entry.kind === kind) && matchesQuery(query, entry.title, entry.locked ? '' : entry.body)
    )
    .sort((left, right) => right.occurredAt - left.occurredAt);
  const years = groupYears(entries);
  const subtitle = members.length === 0 ? '还没有家人' : `家里 · ${members.length} 位家人`;

  useEffect(() => {
    if (!search.id) {
      return;
    }
    void navigate({ to: PATH.growthEntry, params: { entryId: search.id }, replace: true });
  }, [navigate, search.id]);

  useEffect(() => {
    if (!search.member || membersState.isPending || member) {
      return;
    }
    void navigate({ to: PATH.growth, search: {}, replace: true });
  }, [member, membersState.isPending, navigate, search.member]);

  function stay(extra: { create?: boolean; kind?: GrowthKind; sheet?: 'family' | 'measure' } = {}) {
    void navigate({
      to: PATH.growth,
      search: {
        ...(member ? { member: member.id } : {}),
        ...extra
      }
    });
  }

  function closeSheet() {
    void navigate({
      to: PATH.growth,
      search: member ? { member: member.id } : {},
      replace: true
    });
  }

  if (search.sheet === 'family') {
    return (
      <FamilyView
        onBack={closeSheet}
        onRemoved={(id) => {
          if (id === member?.id) {
            void navigate({ to: PATH.growth, search: {}, replace: true });
          }
        }}
      />
    );
  }

  if (search.create) {
    if (!member) {
      return <FamilyView onBack={() => void navigate({ to: PATH.growth, search: {} })} onRemoved={() => undefined} />;
    }
    return (
      <MobileGrowthEditor
        ownerId={member.id}
        kind={search.kind ?? 'milestone'}
        onClose={() => void navigate({ to: PATH.growth, search: { member: member.id } })}
        onSaved={(entry) => {
          void navigate({ to: PATH.growthEntry, params: { entryId: entry.id } });
        }}
      />
    );
  }

  return (
    <section className={styles.page}>
      <AppBar title="成长" subtitle={subtitle}>
        <IconButton
          label="记一笔"
          onClick={() => stay({ create: true, kind: kind === 'moment' ? 'moment' : 'milestone' })}
        >
          <PlusIcon weight="regular" />
        </IconButton>
      </AppBar>
      <form
        className={styles.search}
        onSubmit={(event) => {
          event.preventDefault();
        }}
      >
        <input
          type="search"
          value={query}
          placeholder="搜索成长记录"
          aria-label="搜索成长记录"
          onChange={(event) => setQuery(event.target.value)}
        />
      </form>
      <div className={styles.body}>
        {membersState.error || listQuery.error ? (
          <p className={styles.error}>{errorMessage(membersState.error ?? listQuery.error)}</p>
        ) : null}
        {members.length === 0 && !membersState.isPending ? (
          <button type="button" className={styles.empty} onClick={() => stay({ sheet: 'family' })}>
            先添加一位家人
          </button>
        ) : null}
        {members.length > 0 ? (
          <div className={styles.people}>
            {members.map((person) => (
              <button
                key={person.id}
                type="button"
                className={styles.person}
                data-on={person.id === member?.id ? 'true' : undefined}
                onClick={() => void navigate({ to: PATH.growth, search: { member: person.id } })}
              >
                <MemberFace
                  name={person.name}
                  imageId={person.avatar}
                  owner={{ owner: 'member', ownerId: person.id }}
                />
                <span>{person.name}</span>
              </button>
            ))}
            <button type="button" className={styles.person} onClick={() => stay({ sheet: 'family' })}>
              <span className={styles.add}>+</span>
              <span>家人</span>
            </button>
          </div>
        ) : null}
        {member ? (
          <section className={styles.card}>
            <div>
              <strong>{member.name}</strong>
              <p>
                {[member.relation, member.birthday != null ? ageLabel(member.birthday) : '']
                  .filter(Boolean)
                  .join(' · ')}
              </p>
            </div>
            <button type="button" onClick={() => stay({ sheet: 'measure' })}>
              {latest ? `${cmText(latest.heightMm)} · ${kgText(latest.weightG)}` : '记一次身高体重'}
            </button>
          </section>
        ) : null}
        <div className={styles.segments} role="tablist" aria-label="记录类型">
          <SegmentButton selected={kind === 'all'} onClick={() => setKind('all')}>
            全部
          </SegmentButton>
          <SegmentButton selected={kind === 'milestone'} onClick={() => setKind('milestone')}>
            里程碑
          </SegmentButton>
          <SegmentButton selected={kind === 'moment'} onClick={() => setKind('moment')}>
            瞬间
          </SegmentButton>
        </div>
        {entries.length === 0 && !listQuery.isPending && member ? (
          <p className={styles.note}>{query.trim() ? '没有匹配的记录。' : '还没有记录。'}</p>
        ) : null}
        {years.map((group) => (
          <section key={group.year}>
            <h2 className={styles.year}>{group.year}</h2>
            <ul className={styles.list}>
              {group.entries.map((entry) => (
                <li key={entry.id}>
                  <button
                    type="button"
                    onClick={() => void navigate({ to: PATH.growthEntry, params: { entryId: entry.id } })}
                  >
                    <span className={styles.when}>{formatMonthDay(entry.occurredAt)}</span>
                    <span>
                      <strong>{entry.title}</strong>
                      <span>{journalExcerpt(entry.body, entry.locked)}</span>
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          </section>
        ))}
      </div>
      {search.sheet === 'measure' && member ? (
        <MeasureForm memberId={member.id} measure={null} onClose={closeSheet} />
      ) : null}
    </section>
  );
}

function groupYears<T extends { occurredAt: number }>(entries: T[]) {
  const groups: { year: number; entries: T[] }[] = [];
  for (const entry of entries) {
    const year = new Date(entry.occurredAt).getFullYear();
    const found = groups.find((group) => group.year === year);
    if (found) {
      found.entries.push(entry);
    } else {
      groups.push({ year, entries: [entry] });
    }
  }
  return groups;
}
