import {
  BookOpenIcon,
  CalendarBlankIcon,
  CameraIcon,
  ClockCounterClockwiseIcon,
  FireIcon,
  FlagIcon,
  HourglassIcon,
  LightningIcon,
  ListChecksIcon,
  LockIcon,
  NotebookIcon,
  PencilLineIcon,
  PencilSimpleIcon,
  QuotesIcon
} from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useState } from 'react';
import type { Plan } from '@/bridge/plan';
import type { TimelineItem, TimelineKind } from '@/bridge/timeline';
import { CompleteDialog } from '@/components/complete-dialog';
import { PlanCheck } from '@/components/plan-check';
import { usePageSearch } from '@/layout/page-search';
import { calendarCaption, calendarRangeQuery } from '@/query/calendar/query';
import { useCompletePlan } from '@/query/plan/complete';
import { planMeta, todayCompletion, todayPlanPanel } from '@/query/plan/group';
import { planListQuery } from '@/query/plan/query';
import { taskListQuery } from '@/query/task/query';
import { timelineListQuery } from '@/query/timeline/query';
import { PATH } from '@/router/path';
import { addLocalDays, formatDayHeading, formatFeedClock, startOfLocalDay, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import { matchesQuery } from '@/utils/search';
import styles from './page.module.css';

const kindMeta: Record<TimelineKind, { label: string; tone: string; Icon: typeof ListChecksIcon }> = {
  plan: { label: '计划', tone: 'plan', Icon: ListChecksIcon },
  diary: { label: '日记', tone: 'journal', Icon: NotebookIcon },
  spark: { label: '灵感', tone: 'journal', Icon: LightningIcon },
  writing: { label: '写作', tone: 'journal', Icon: PencilLineIcon },
  milestone: { label: '里程碑', tone: 'growth', Icon: FlagIcon },
  moment: { label: '瞬间', tone: 'growth', Icon: CameraIcon },
  excerpt: { label: '书摘', tone: 'library', Icon: QuotesIcon },
  note: { label: '笔记', tone: 'library', Icon: BookOpenIcon }
};

const kindOrder: TimelineKind[] = ['diary', 'spark', 'writing', 'milestone', 'moment', 'excerpt', 'note', 'plan'];

export function TodayPage() {
  const navigate = useNavigate();
  const todayStart = startOfLocalDay();
  const tomorrow = addLocalDays(todayStart, 1);
  const streakFrom = addLocalDays(todayStart, -59);
  const plansQuery = useQuery(planListQuery);
  const feedQuery = useQuery(timelineListQuery(todayStart, tomorrow));
  const streakQuery = useQuery(timelineListQuery(streakFrom, tomorrow));
  const tasksQuery = useQuery(taskListQuery);
  const todayKey = toDateInputValue(todayStart);
  const calendarQuery = useQuery(calendarRangeQuery(todayKey, todayKey));
  const [completing, setCompleting] = useState<Plan | null>(null);
  const query = usePageSearch('搜索今天');
  const complete = useCompletePlan(() => setCompleting(null));

  const plans = plansQuery.data ?? [];
  const feed = feedQuery.data ?? [];
  const visibleFeed = feed.filter((item) => matchesQuery(query, item.title, item.body));
  const completion = todayCompletion(plans, todayStart);
  const completionRate = completion.total === 0 ? 0 : Math.round((completion.done / completion.total) * 100);
  const panel = todayPlanPanel(plans, todayStart);
  const visiblePanel = panel.filter((plan) => matchesQuery(query, plan.title, plan.result));
  const overdue = panel.filter(
    (plan) => plan.status !== 'done' && plan.scheduledAt != null && plan.scheduledAt < todayStart
  ).length;
  const running =
    tasksQuery.data?.filter((task) => task.status === 'pending' || task.status === 'processing').length ?? 0;
  const error = plansQuery.error ?? feedQuery.error ?? streakQuery.error ?? tasksQuery.error ?? calendarQuery.error;
  const caption = calendarCaption(calendarQuery.data?.days[0]);

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>今天</h1>
        <p className={styles.hint}>
          {formatDayHeading(todayStart)}
          {caption ? ` · ${caption}` : ''}
        </p>
      </header>

      <div className={styles.capture}>
        <PencilSimpleIcon className={styles.statIcon} weight="regular" />
        <span className={styles.prompt}>记点什么…文字、里程碑、书摘、计划，都从这里进</span>
        <button
          type="button"
          className={styles.chip}
          data-tone="journal"
          onClick={() => void navigate({ to: PATH.journal, search: { create: true, kind: 'diary' } })}
        >
          <PencilSimpleIcon className={styles.chipIcon} weight="regular" />
          文字
        </button>
        <button
          type="button"
          className={styles.chip}
          data-tone="growth"
          onClick={() => void navigate({ to: PATH.growth, search: { create: true, kind: 'milestone' } })}
        >
          <FlagIcon className={styles.chipIcon} weight="regular" />
          里程碑
        </button>
        <button
          type="button"
          className={styles.chip}
          data-tone="library"
          onClick={() => void navigate({ to: PATH.library })}
        >
          <QuotesIcon className={styles.chipIcon} weight="regular" />
          书摘
        </button>
        <button
          type="button"
          className={styles.chip}
          data-tone="plan"
          onClick={() => void navigate({ to: PATH.plan, search: { create: true } })}
        >
          <ListChecksIcon className={styles.chipIcon} weight="regular" />
          计划
        </button>
      </div>

      {error ? <p className={styles.error}>{errorMessage(error)}</p> : null}

      <div className={styles.stats}>
        <article className={styles.stat}>
          <ListChecksIcon className={styles.statIcon} weight="regular" />
          <span className={styles.statValue}>
            {completion.done}
            <small> / {completion.total}</small>
          </span>
          <span className={styles.statLabel}>今日计划完成</span>
          <div className={styles.progress}>
            <i style={{ width: `${completionRate}%` }} />
          </div>
        </article>
        <article className={styles.stat}>
          <NotebookIcon className={styles.statIcon} weight="regular" />
          <span className={styles.statValue}>
            {feed.length} <small>条</small>
          </span>
          <span className={styles.statLabel}>今天记录{recordSummary(feed) ? ` · ${recordSummary(feed)}` : ''}</span>
        </article>
        <article className={styles.stat}>
          <FireIcon className={styles.statIcon} weight="regular" />
          <span className={styles.statValue}>
            {streakCount(streakQuery.data ?? [], todayStart)} <small>天</small>
          </span>
          <span className={styles.statLabel}>连续记录</span>
        </article>
        <article className={styles.stat}>
          <HourglassIcon className={styles.statIcon} weight="regular" />
          <span className={styles.statValue}>
            {running} <small>运行中</small>
          </span>
          <span className={styles.statLabel}>后台处理</span>
        </article>
      </div>

      <div className={styles.split}>
        <article className={styles.card}>
          <h2 className={styles.cardTitle}>
            <ClockCounterClockwiseIcon className={styles.icon} weight="regular" />
            今日动态
            <span className={styles.extra}>按时间混排</span>
          </h2>
          {feedQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}
          {!feedQuery.isPending && visibleFeed.length === 0 ? (
            <p className={styles.empty}>{query.trim() ? '没有匹配的记录。' : '今天还没有新的记录。'}</p>
          ) : null}
          {visibleFeed.map((item) => (
            <FeedRow key={`${item.kind}-${item.id}`} item={item} />
          ))}
        </article>
        <article className={styles.card}>
          <h2 className={styles.cardTitle}>
            <CalendarBlankIcon className={styles.icon} weight="regular" />
            今日计划
            {overdue > 0 ? <span className={styles.extra}>{overdue} 项逾期</span> : null}
          </h2>
          {plansQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}
          {!plansQuery.isPending && visiblePanel.length === 0 ? (
            <p className={styles.empty}>{query.trim() ? '没有匹配的计划。' : '今天还没有计划。'}</p>
          ) : null}
          {visiblePanel.map((plan) => (
            <PlanCheck
              key={plan.id}
              plan={plan}
              meta={planMeta(plan, todayStart)}
              onComplete={setCompleting}
              onOpen={(item) => void navigate({ to: PATH.plan, search: { id: item.id } })}
            />
          ))}
        </article>
      </div>
      <CompleteDialog
        plan={completing}
        pending={complete.pending}
        error={complete.error ? errorMessage(complete.error) : ''}
        onSubmit={(result) => {
          if (completing) {
            complete.submit(completing.id, result);
          }
        }}
        onClose={() => setCompleting(null)}
      />
    </section>
  );
}

function FeedRow({ item }: { item: TimelineItem }) {
  const navigate = useNavigate();
  const meta = kindMeta[item.kind];
  const Icon = meta.Icon;
  const text = item.title.trim() || item.body.trim() || '未命名';
  const body = (
    <>
      <span className={styles.kind} data-tone={meta.tone}>
        <Icon className={styles.icon} weight="regular" />
        {meta.label}
      </span>
      <span className={styles.feedText}>{text}</span>
      {item.locked ? <LockIcon className={styles.lock} weight="regular" /> : null}
      <time className={styles.clock}>{formatFeedClock(item.occurredAt)}</time>
    </>
  );

  if (item.kind === 'plan') {
    return (
      <button
        type="button"
        className={styles.feed}
        onClick={() => void navigate({ to: PATH.plan, search: { id: item.id } })}
      >
        {body}
      </button>
    );
  }
  if (item.kind === 'diary' || item.kind === 'spark' || item.kind === 'writing') {
    return (
      <button
        type="button"
        className={styles.feed}
        onClick={() => void navigate({ to: PATH.journalEntry, params: { entryId: item.id } })}
      >
        {body}
      </button>
    );
  }
  if (item.kind === 'milestone' || item.kind === 'moment') {
    return (
      <button type="button" className={styles.feed} onClick={() => void navigate({ to: PATH.growth })}>
        {body}
      </button>
    );
  }
  if (item.kind === 'excerpt' || item.kind === 'note') {
    return (
      <button type="button" className={styles.feed} onClick={() => void navigate({ to: PATH.library })}>
        {body}
      </button>
    );
  }
  return <div className={styles.feedStatic}>{body}</div>;
}

function recordSummary(items: TimelineItem[]): string {
  const counts = new Map<TimelineKind, number>();
  for (const item of items) {
    counts.set(item.kind, (counts.get(item.kind) ?? 0) + 1);
  }
  return kindOrder
    .filter((kind) => counts.get(kind))
    .map((kind) => `${kindMeta[kind].label}${counts.get(kind)}`)
    .join(' ');
}

function streakCount(items: { occurredAt: number }[], todayStart: number): number {
  const days = new Set(items.map((item) => startOfLocalDay(item.occurredAt)));
  let cursor = days.has(todayStart) ? todayStart : addLocalDays(todayStart, -1);
  let count = 0;
  while (days.has(cursor)) {
    count += 1;
    cursor = addLocalDays(cursor, -1);
  }
  return count;
}
