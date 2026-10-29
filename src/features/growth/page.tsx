import { CameraIcon, CaretLeftIcon, FlagIcon, PlayIcon, PlusIcon, SquaresFourIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import type { GrowthEntry, GrowthKind, GrowthMeasure } from '@/bridge/growth';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { SegmentButton } from '@/components/button/segment';
import { firstImageId, firstVideoId, UploadThumb } from '@/components/markdown';
import { Scroll } from '@/components/scroll';
import { useJournalNames } from '@/features/journal/names';
import { journalDate, journalExcerpt, lookupNames } from '@/features/journal/text';
import { usePageSearch } from '@/hooks/search';
import { growthListQuery, growthMeasureListQuery } from '@/query/growth/query';
import { useMembers } from '@/query/member/use-members';
import { PATH } from '@/router/path';
import { ageLabel } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import { matchesQuery } from '@/utils/search';
import { GrowthEditor } from './components/editor';
import { MemberFace } from './components/face';
import { FamilyView } from './components/family';
import { MeasureForm } from './components/measure-form';
import styles from './page.module.css';
import { cardAge, cmText, groupChapters, isYoung, kgText } from './text';

const segments: { kind: GrowthKind | 'all'; label: string; Icon: typeof FlagIcon }[] = [
  { kind: 'all', label: '全部', Icon: SquaresFourIcon },
  { kind: 'milestone', label: '里程碑', Icon: FlagIcon },
  { kind: 'moment', label: '瞬间', Icon: CameraIcon }
];

/** 成长。家里看全家，点开一个人看轨迹。 */
export function GrowthPage() {
  const navigate = useNavigate();
  const search = useSearch({ from: '/growth' });
  const membersState = useMembers();
  const members = membersState.members;
  const names = useJournalNames();
  const query = usePageSearch('搜索成长');
  const listQuery = useQuery(growthListQuery());
  const [kind, setKind] = useState<GrowthKind | 'all'>(search.kind ?? 'all');
  const [creating, setCreating] = useState(false);
  const [draftKind, setDraftKind] = useState<GrowthKind>(search.kind ?? 'milestone');
  const [family, setFamily] = useState(false);
  const [measure, setMeasure] = useState<GrowthMeasure | 'new' | null>(null);
  const member = members.find((item) => item.id === search.member) ?? null;
  const measuresQuery = useQuery({
    ...growthMeasureListQuery(member?.id ?? ''),
    enabled: member != null
  });
  const records = [...(listQuery.data ?? [])].sort((left, right) => right.occurredAt - left.occurredAt);

  useEffect(() => {
    if (search.id) {
      void navigate({ to: PATH.growthEntry, params: { entryId: search.id }, replace: true });
      return;
    }
    if (search.kind) {
      setKind(search.kind);
    }
    if (!search.create && !search.kind) {
      return;
    }
    if (search.create) {
      if (membersState.isPending) {
        return;
      }
      setDraftKind(search.kind ?? 'milestone');
      if (members.length === 0) {
        setFamily(true);
      } else {
        setCreating(true);
      }
    }
    void navigate({
      to: PATH.growth,
      search: search.member ? { member: search.member } : {},
      replace: true
    });
  }, [members.length, membersState.isPending, navigate, search.create, search.id, search.kind, search.member]);

  useEffect(() => {
    if (!search.member || membersState.isPending) {
      return;
    }
    if (!members.some((item) => item.id === search.member)) {
      void navigate({ to: PATH.growth, search: {}, replace: true });
    }
  }, [members, membersState.isPending, navigate, search.member]);

  function visible(entry: GrowthEntry) {
    return matchesQuery(query, entry.title, entry.locked ? '' : entry.body);
  }

  function openCreate() {
    if (members.length === 0) {
      setFamily(true);
      return;
    }
    setDraftKind(kind === 'moment' ? 'moment' : 'milestone');
    setCreating(true);
  }

  function openEntry(id: string) {
    void navigate({ to: PATH.growthEntry, params: { entryId: id } });
  }

  function entryMeta(entry: GrowthEntry, withName: boolean) {
    const person = members.find((item) => item.id === entry.memberId);
    const young = isYoung(person?.birthday ?? null);
    return [
      withName ? (person?.name ?? '') : '',
      cardAge(person?.birthday ?? null, entry.occurredAt, young),
      ...lookupNames(entry.placeIds, names.places),
      ...lookupNames(
        entry.memberIds.filter((id) => id !== entry.memberId),
        names.members
      ),
      ...lookupNames(entry.tagIds, names.tags)
    ]
      .filter((item) => item.length > 0)
      .join(' · ');
  }

  if (creating) {
    const ownerId = member?.id ?? members[0]?.id;
    if (!ownerId) {
      return (
        <FamilyView
          onBack={() => setCreating(false)}
          onRemoved={(id) => {
            if (search.member === id) {
              void navigate({ to: PATH.growth, search: {} });
            }
          }}
        />
      );
    }
    return (
      <GrowthEditor
        entry={null}
        ownerId={ownerId}
        kind={draftKind}
        onClose={() => setCreating(false)}
        onSaved={(entry) => {
          setCreating(false);
          void navigate({ to: PATH.growthEntry, params: { entryId: entry.id } });
        }}
      />
    );
  }

  if (family) {
    return (
      <FamilyView
        onBack={() => setFamily(false)}
        onRemoved={(id) => {
          if (search.member === id) {
            void navigate({ to: PATH.growth, search: {} });
          }
        }}
      />
    );
  }

  const waitingMember = search.member != null && membersState.isPending && member == null;

  return (
    <section className={styles.page}>
      {waitingMember ? <p className={styles.note}>正在读取…</p> : null}
      {!waitingMember && member ? (
        <Timeline
          memberId={member.id}
          memberName={member.name}
          avatar={member.avatar}
          who={[member.relation, memberLine(member.birthday)].filter(Boolean).join(' · ')}
          kind={kind}
          entries={records.filter(
            (entry) => entry.memberId === member.id && (kind === 'all' || entry.kind === kind) && visible(entry)
          )}
          birthday={member.birthday}
          pending={listQuery.isPending}
          error={listQuery.error ? errorMessage(listQuery.error) : ''}
          searching={query.trim().length > 0}
          measures={measuresQuery.data ?? []}
          measuresPending={measuresQuery.isPending}
          measuresError={measuresQuery.error ? errorMessage(measuresQuery.error) : ''}
          onBack={() => void navigate({ to: PATH.growth, search: {} })}
          onKind={(next) => {
            setKind(next);
            if (next !== 'all') {
              setMeasure(null);
            }
          }}
          onFamily={() => setFamily(true)}
          onCreate={openCreate}
          onOpen={openEntry}
          meta={(entry) => entryMeta(entry, false)}
          onMeasure={setMeasure}
        />
      ) : null}
      {!waitingMember && !member ? (
        <Home
          members={members}
          records={records}
          pending={listQuery.isPending || membersState.isPending}
          error={listQuery.error || membersState.error ? errorMessage(listQuery.error ?? membersState.error) : ''}
          searching={query.trim().length > 0}
          onFamily={() => setFamily(true)}
          onCreate={openCreate}
          onOpen={openEntry}
          onPerson={(id) => void navigate({ to: PATH.growth, search: { member: id } })}
          meta={(entry) => entryMeta(entry, true)}
          visible={visible}
        />
      ) : null}
      {member && measure !== null ? (
        <MeasureForm
          memberId={member.id}
          measure={measure === 'new' ? null : measure}
          onClose={() => setMeasure(null)}
        />
      ) : null}
    </section>
  );
}

function memberLine(birthday: number | null) {
  if (birthday == null) {
    return '还没有生日';
  }
  const age = ageLabel(birthday);
  return age ? `现在 ${age}` : '';
}

function Home({
  members,
  records,
  pending,
  error,
  searching,
  onFamily,
  onCreate,
  onOpen,
  onPerson,
  meta,
  visible
}: {
  members: { id: string; name: string; relation: string; birthday: number | null; avatar: string }[];
  records: GrowthEntry[];
  pending: boolean;
  error: string;
  searching: boolean;
  onFamily: () => void;
  onCreate: () => void;
  onOpen: (id: string) => void;
  onPerson: (id: string) => void;
  meta: (entry: GrowthEntry) => string;
  visible: (entry: GrowthEntry) => boolean;
}) {
  const shown = records.filter(visible);
  const hero = shown.find((entry) => entry.highlight) ?? null;
  const heroSide = hero == null ? { photo: '', video: false } : entrySide(hero);
  const feed = shown.filter((entry) => entry.id !== hero?.id).slice(0, 6);

  return (
    <>
      <header className={styles.head}>
        <h1 className={styles.title}>家里</h1>
        <span className={styles.spacer} />
        <Button variant="outlined" onClick={onFamily}>
          家人
        </Button>
        <Button onClick={onCreate}>
          <PlusIcon weight="regular" />
          记一笔
        </Button>
      </header>
      <Scroll className={styles.scroll}>
        <div className={styles.body}>
          {error ? <p className={styles.error}>{error}</p> : null}
          {pending ? <p className={styles.note}>正在读取…</p> : null}
          {!pending && members.length === 0 ? (
            <>
              <p className={styles.empty}>先添加一位家人。</p>
              <Button onClick={onFamily}>添加家人</Button>
            </>
          ) : null}
          {hero ? (
            <button
              type="button"
              className={styles.hero}
              data-photo={heroSide.photo || heroSide.video ? 'true' : undefined}
              onClick={() => onOpen(hero.id)}
            >
              <DateMark time={hero.occurredAt} />
              <span className={styles.heroMain}>
                <span className={styles.kicker}>最近的高光</span>
                <span className={styles.heroTitle}>{hero.title}</span>
                <span className={styles.excerpt}>{journalExcerpt(hero.body, hero.locked)}</span>
                <span className={styles.quiet}>{meta(hero)}</span>
              </span>
              <EntryMedia entry={hero} side={heroSide} />
            </button>
          ) : null}
          {members.length > 0 ? (
            <>
              <h2 className={styles.sectionTitle}>家人</h2>
              <div className={styles.people}>
                {members.map((person) => {
                  const own = records.filter((entry) => entry.memberId === person.id);
                  const age = person.birthday != null ? ageLabel(person.birthday) : '';
                  const personMeta = [person.relation, age].filter(Boolean).join(' · ');
                  return (
                    <button key={person.id} type="button" className={styles.person} onClick={() => onPerson(person.id)}>
                      <MemberFace
                        name={person.name}
                        imageId={person.avatar}
                        owner={{ owner: 'member', ownerId: person.id }}
                      />
                      <span className={styles.personCopy}>
                        <span className={styles.personName}>{person.name}</span>
                        {personMeta ? <span className={styles.personMeta}>{personMeta}</span> : null}
                        <span className={styles.personCount}>
                          {own.length === 0 ? '还没有记录' : `${own.length} 条`}
                        </span>
                        {own[0] ? <span className={styles.personLatest}>{own[0].title}</span> : null}
                      </span>
                    </button>
                  );
                })}
              </div>
            </>
          ) : null}
          {members.length > 0 && !pending && shown.length === 0 ? (
            <p className={styles.empty}>{searching ? '没有匹配的记录。' : '家里还没有记录。'}</p>
          ) : null}
          {feed.length > 0 ? (
            <>
              <h2 className={styles.sectionTitle}>最近</h2>
              <ul className={styles.list}>
                {feed.map((entry) => (
                  <EntryCard key={entry.id} entry={entry} meta={meta(entry)} showKind onOpen={() => onOpen(entry.id)} />
                ))}
              </ul>
            </>
          ) : null}
        </div>
      </Scroll>
    </>
  );
}

function Timeline({
  memberId,
  memberName,
  avatar,
  who,
  kind,
  entries,
  birthday,
  pending,
  error,
  searching,
  measures,
  measuresPending,
  measuresError,
  onBack,
  onKind,
  onFamily,
  onCreate,
  onOpen,
  meta,
  onMeasure
}: {
  memberId: string;
  memberName: string;
  avatar: string;
  who: string;
  kind: GrowthKind | 'all';
  entries: GrowthEntry[];
  birthday: number | null;
  pending: boolean;
  error: string;
  searching: boolean;
  measures: GrowthMeasure[];
  measuresPending: boolean;
  measuresError: string;
  onBack: () => void;
  onKind: (kind: GrowthKind | 'all') => void;
  onFamily: () => void;
  onCreate: () => void;
  onOpen: (id: string) => void;
  meta: (entry: GrowthEntry) => string;
  onMeasure: (measure: GrowthMeasure | 'new') => void;
}) {
  const chapters = groupChapters(entries, birthday, isYoung(birthday));
  const latest = measures[0];

  return (
    <>
      <header className={styles.headStack}>
        <div className={styles.head}>
          <IconButton label="返回家里" onClick={onBack}>
            <CaretLeftIcon weight="regular" />
          </IconButton>
          <div className={styles.who}>
            <MemberFace name={memberName} imageId={avatar} owner={{ owner: 'member', ownerId: memberId }} />
            <div className={styles.whoCopy}>
              <strong>{memberName}</strong>
              {who ? <span>{who}</span> : null}
            </div>
          </div>
          <span className={styles.spacer} />
          <Button variant="outlined" onClick={onFamily}>
            编辑档案
          </Button>
          <Button onClick={onCreate}>
            <PlusIcon weight="regular" />
            记一笔
          </Button>
        </div>
        <div className={styles.segments} role="tablist" aria-label="成长类型">
          {segments.map((item) => (
            <SegmentButton key={item.kind} selected={kind === item.kind} onClick={() => onKind(item.kind)}>
              <item.Icon weight="regular" />
              {item.label}
            </SegmentButton>
          ))}
        </div>
      </header>
      <Scroll className={styles.scroll}>
        <div className={styles.body}>
          {error ? <p className={styles.error}>{error}</p> : null}
          {kind === 'all' ? (
            <section className={styles.measure}>
              <div className={styles.measureTop}>
                <h2>生长</h2>
                <Button variant="text" onClick={() => onMeasure('new')}>
                  记一次
                </Button>
              </div>
              {measuresError ? <p className={styles.error}>{measuresError}</p> : null}
              {measuresPending ? <p className={styles.note}>正在读取…</p> : null}
              {latest ? (
                <p className={styles.measureNow}>
                  <strong>
                    {cmText(latest.heightMm)} <span>厘米</span>
                  </strong>
                  <strong>
                    {kgText(latest.weightG)} <span>千克</span>
                  </strong>
                </p>
              ) : null}
              {!measuresPending && measures.length === 0 ? <p className={styles.note}>记下身高和体重。</p> : null}
              {measures.length > 0 ? (
                <ul className={styles.measureList}>
                  {measures.map((item) => (
                    <li key={item.id}>
                      <button type="button" className={styles.measureRow} onClick={() => onMeasure(item)}>
                        <span>{journalDate(item.occurredAt).full}</span>
                        <span>{cmText(item.heightMm)} 厘米</span>
                        <span>{kgText(item.weightG)} 千克</span>
                      </button>
                    </li>
                  ))}
                </ul>
              ) : null}
            </section>
          ) : null}
          {pending ? <p className={styles.note}>正在读取…</p> : null}
          {!pending && entries.length === 0 ? (
            <p className={styles.empty}>{searching ? '没有匹配的记录。' : '这条轨迹里还没有这种记录。'}</p>
          ) : null}
          {chapters.length > 0 ? (
            <div className={styles.chapters}>
              {chapters.map((chapter) => (
                <section key={chapter.key} className={styles.chapter}>
                  <h2 className={styles.chapterTitle}>
                    {chapter.title}
                    {chapter.subtitle ? <span>{chapter.subtitle}</span> : null}
                  </h2>
                  <ul className={styles.list}>
                    {chapter.entries.map((entry) => (
                      <EntryCard
                        key={entry.id}
                        entry={entry}
                        meta={meta(entry)}
                        showKind={kind === 'all'}
                        onOpen={() => onOpen(entry.id)}
                      />
                    ))}
                  </ul>
                </section>
              ))}
            </div>
          ) : null}
        </div>
      </Scroll>
    </>
  );
}

function entrySide(entry: GrowthEntry) {
  if (entry.locked) {
    return { photo: '', video: false };
  }
  const photo = firstImageId(entry.body);
  return { photo, video: photo === '' && firstVideoId(entry.body) !== '' };
}

function EntryMedia({ entry, side }: { entry: GrowthEntry; side: { photo: string; video: boolean } }) {
  if (side.photo) {
    return <UploadThumb id={side.photo} owner="growth_entry" ownerId={entry.id} className={styles.thumb} />;
  }
  if (!side.video) {
    return null;
  }
  return (
    <span className={styles.thumb} data-video="true">
      <PlayIcon className={styles.play} weight="fill" />
    </span>
  );
}

function DateMark({ time }: { time: number }) {
  const date = journalDate(time);
  return (
    <span className={styles.date}>
      <span className={styles.day}>{date.day}</span>
      <span className={styles.month}>{date.month}</span>
      <span className={styles.week}>{date.week}</span>
    </span>
  );
}

function EntryCard({
  entry,
  meta,
  showKind,
  onOpen
}: {
  entry: GrowthEntry;
  meta: string;
  showKind: boolean;
  onOpen: () => void;
}) {
  const side = entrySide(entry);
  return (
    <li>
      <button
        type="button"
        className={styles.card}
        data-photo={side.photo || side.video ? 'true' : undefined}
        onClick={onOpen}
      >
        <DateMark time={entry.occurredAt} />
        <span className={styles.cardMain}>
          <span className={styles.cardTop}>
            <span className={styles.cardTitle}>{entry.title}</span>
            {entry.highlight || entry.locked ? (
              <span className={styles.badges}>
                {entry.highlight ? (
                  <span className={styles.badge} data-tone="highlight">
                    高光
                  </span>
                ) : null}
                {entry.locked ? <span className={styles.badge}>已封存</span> : null}
              </span>
            ) : null}
          </span>
          <span className={styles.excerpt}>{journalExcerpt(entry.body, entry.locked)}</span>
          <span className={styles.cardFoot}>
            {showKind ? <span className={styles.quiet}>{entry.kind === 'milestone' ? '里程碑' : '瞬间'}</span> : null}
            {meta ? <span className={styles.quiet}>{meta}</span> : null}
          </span>
        </span>
        <EntryMedia entry={entry} side={side} />
      </button>
    </li>
  );
}
