import { GearIcon, MagnifyingGlassIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { GrowthKind } from '@/bridge/growth';
import type { Member } from '@/bridge/member';
import { IconButton } from '@/components/button/icon';
import { SegmentButton } from '@/components/button/segment';
import { MemberFace } from '@/features/growth/components/face';
import { FamilyView, MemberEditPage } from '@/features/growth/components/family';
import { MobileGrowthEditor } from '@/features/growth/mobile/editor';
import { journalExcerpt } from '@/features/journal/text';
import { AppBar } from '@/layout/mobile/app-bar';
import { popRoute } from '@/layout/mobile/back';
import { Fab } from '@/layout/mobile/fab';
import { growthListQuery } from '@/query/growth/query';
import { useMembers } from '@/query/member/use-members';
import { PATH } from '@/router/path';
import { formatMonthDay } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import { matchesQuery } from '@/utils/search';
import styles from './page.module.css';

/** 移动端成长：家人头像、按年排的记录。 */
export function MobileGrowthPage() {
  const { t } = useTranslation(['growth', 'common']);
  const navigate = useNavigate();
  const search = useSearch({ from: '/growth' });
  const membersState = useMembers();
  const members = membersState.members;
  const member = members.find((item) => item.id === search.member) ?? members[0] ?? null;
  const [kind, setKind] = useState<GrowthKind | 'all'>('all');
  const [finding, setFinding] = useState(false);
  const [query, setQuery] = useState('');
  const listQuery = useQuery(growthListQuery(member?.id));
  const entries = [...(listQuery.data ?? [])]
    .filter(
      (entry) =>
        (kind === 'all' || entry.kind === kind) && matchesQuery(query, entry.title, entry.locked ? '' : entry.body)
    )
    .sort((left, right) => right.occurredAt - left.occurredAt);
  const years = groupYears(entries);
  const subtitle = members.length === 0 ? t('page.noFamily') : t('page.familyCount', { count: members.length });

  useEffect(() => {
    if (!search.id) {
      return;
    }
    void navigate({ to: PATH.growthEntry, params: { entryId: search.id }, replace: true });
  }, [navigate, search.id]);

  useEffect(() => {
    if (!search.member || membersState.isPending) {
      return;
    }
    if (members.some((item) => item.id === search.member)) {
      return;
    }
    if (search.edit === search.member) {
      return;
    }
    void navigate({
      to: PATH.growth,
      search: search.sheet ? { sheet: search.sheet } : {},
      replace: true
    });
  }, [members, membersState.isPending, navigate, search.edit, search.member, search.sheet]);

  function stay(extra: { create?: boolean; kind?: GrowthKind; sheet?: 'family' | 'measure'; edit?: string } = {}) {
    void navigate({
      to: PATH.growth,
      search: {
        ...(member ? { member: member.id } : {}),
        ...extra
      }
    });
  }

  function openMember(target: Member | 'new') {
    void navigate({
      to: PATH.growth,
      search: {
        sheet: 'family',
        edit: target === 'new' ? 'new' : target.id,
        ...(target === 'new' ? (member ? { member: member.id } : {}) : { member: target.id })
      }
    });
  }

  function closeSheet() {
    popRoute(() => {
      void navigate({
        to: PATH.growth,
        search: member ? { member: member.id } : {},
        replace: true
      });
    });
  }

  if (search.edit) {
    const editing = search.edit === 'new' ? null : (members.find((item) => item.id === search.edit) ?? null);
    if (search.edit !== 'new' && !editing) {
      return (
        <section className={styles.page}>
          <p className={styles.note}>{membersState.isPending ? t('page.loading') : t('page.memberNotFound')}</p>
        </section>
      );
    }
    return (
      <MemberEditPage
        key={search.edit}
        member={editing}
        onClose={closeSheet}
        onRemoved={() => undefined}
        onCreated={(saved) => {
          void navigate({
            to: PATH.growth,
            search: {
              sheet: 'family',
              edit: saved.id,
              member: member?.id ?? saved.id
            },
            replace: true
          });
        }}
      />
    );
  }

  if (search.sheet === 'family' || search.sheet === 'measure') {
    return (
      <FamilyView
        onBack={closeSheet}
        onEdit={openMember}
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
      return (
        <FamilyView
          onBack={() => popRoute(() => void navigate({ to: PATH.growth, search: {}, replace: true }))}
          onRemoved={() => undefined}
        />
      );
    }
    return (
      <MobileGrowthEditor
        ownerId={member.id}
        kind={search.kind ?? 'moment'}
        onClose={() => popRoute(() => void navigate({ to: PATH.growth, search: { member: member.id }, replace: true }))}
        onSaved={(entry) => {
          void navigate({ to: PATH.growthEntry, params: { entryId: entry.id }, replace: true });
        }}
      />
    );
  }

  return (
    <section className={styles.page}>
      <AppBar title={t('page.screenTitle')} subtitle={subtitle}>
        <IconButton
          label={t('page.search')}
          onClick={() =>
            setFinding((current) => {
              if (current) {
                setQuery('');
              }
              return !current;
            })
          }
        >
          <MagnifyingGlassIcon weight="regular" />
        </IconButton>
        <IconButton label={t('page.familySettings')} onClick={() => stay({ sheet: 'family' })}>
          <GearIcon weight="regular" />
        </IconButton>
      </AppBar>
      {finding ? (
        <form
          className={styles.search}
          onSubmit={(event) => {
            event.preventDefault();
          }}
        >
          <input
            type="search"
            value={query}
            placeholder={t('page.search')}
            aria-label={t('page.search')}
            onChange={(event) => setQuery(event.target.value)}
          />
          <button
            type="button"
            onClick={() => {
              setQuery('');
              setFinding(false);
            }}
          >
            {t('common:button.cancel')}
          </button>
        </form>
      ) : null}
      <div className={styles.body}>
        {membersState.error || listQuery.error ? (
          <p className={styles.error}>{errorMessage(membersState.error ?? listQuery.error)}</p>
        ) : null}
        {members.length > 0 ? (
          <div className={styles.people}>
            {members.map((person) => (
              <PersonButton
                key={person.id}
                person={person}
                selected={person.id === member?.id}
                onSelect={() => void navigate({ to: PATH.growth, search: { member: person.id }, replace: true })}
                onEdit={() => openMember(person)}
              />
            ))}
          </div>
        ) : null}
        <div className={styles.segments} role="tablist" aria-label={t('page.kindAria')}>
          <SegmentButton selected={kind === 'all'} onClick={() => setKind('all')}>
            {t('page.all')}
          </SegmentButton>
          <SegmentButton selected={kind === 'moment'} onClick={() => setKind('moment')}>
            {t('page.moment')}
          </SegmentButton>
          <SegmentButton selected={kind === 'milestone'} onClick={() => setKind('milestone')}>
            {t('page.milestone')}
          </SegmentButton>
        </div>
        {entries.length === 0 && !listQuery.isPending && member ? (
          <p className={styles.note}>{query.trim() ? t('page.emptySearch') : t('page.emptyMobile')}</p>
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
      {member ? (
        <Fab
          label={t('page.newEntry')}
          onClick={() => stay({ create: true, kind: kind === 'milestone' ? 'milestone' : 'moment' })}
        />
      ) : null}
    </section>
  );
}

function PersonButton({
  person,
  selected,
  onSelect,
  onEdit
}: {
  person: Member;
  selected: boolean;
  onSelect: () => void;
  onEdit: () => void;
}) {
  const suppressClick = useRef(false);

  return (
    <button
      type="button"
      className={styles.person}
      data-on={selected ? 'true' : undefined}
      onContextMenu={(event) => {
        event.preventDefault();
        suppressClick.current = true;
        onEdit();
      }}
      onPointerDown={(event) => {
        if (event.button !== 0) {
          return;
        }
        const pointerId = event.pointerId;
        const originX = event.clientX;
        const originY = event.clientY;
        let fired = false;
        const timer = window.setTimeout(() => {
          fired = true;
          onEdit();
        }, 480);
        function stop() {
          window.clearTimeout(timer);
          window.removeEventListener('pointermove', move);
          window.removeEventListener('pointerup', end);
          window.removeEventListener('pointercancel', end);
        }
        function move(ev: PointerEvent) {
          if (ev.pointerId !== pointerId) {
            return;
          }
          if (Math.hypot(ev.clientX - originX, ev.clientY - originY) > 8) {
            stop();
          }
        }
        function end(ev: PointerEvent) {
          if (ev.pointerId !== pointerId) {
            return;
          }
          stop();
          if (fired) {
            suppressClick.current = true;
          }
        }
        window.addEventListener('pointermove', move);
        window.addEventListener('pointerup', end);
        window.addEventListener('pointercancel', end);
      }}
      onClick={() => {
        if (suppressClick.current) {
          suppressClick.current = false;
          return;
        }
        onSelect();
      }}
    >
      <MemberFace name={person.name} imageId={person.avatar} owner={{ owner: 'member', ownerId: person.id }} />
      <span>{person.name}</span>
    </button>
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
