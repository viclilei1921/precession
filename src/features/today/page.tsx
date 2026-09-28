import {
  BookOpen,
  CalendarBlank,
  Camera,
  ClockCounterClockwise,
  Fire,
  Flag,
  Hourglass,
  Lightning,
  ListChecks,
  Lock,
  Notebook,
  PencilLine,
  PencilSimple,
  Quotes
} from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useState } from 'react';
import type { Plan } from '@/bridge/plan';
import type { TimelineItem, TimelineKind } from '@/bridge/timeline';
import { PATH } from '@/router/path';
import { CompleteDialog } from '@/shared/data/plan/complete-dialog';
import { planMeta, todayCompletion, todayPlanPanel } from '@/shared/data/plan/group';
import { PlanCheck } from '@/shared/data/plan/plan-check';
import { planListQuery } from '@/shared/data/plan/query';
import { timelineListQuery } from '@/shared/data/timeline/query';
import { addLocalDays, formatDayHeading, formatFeedClock, startOfLocalDay } from '@/shared/lib/day';
import { errorMessage } from '@/shared/lib/error';
import styles from './page.module.css';
import { taskListQuery } from './query';

const kindMeta: Record<TimelineKind, { label: string; tone: string; Icon: typeof ListChecks }> = {
  plan: { label: '计划', tone: 'plan', Icon: ListChecks },
  diary: { label: '日记', tone: 'journal', Icon: Notebook },
  spark: { label: '灵感', tone: 'journal', Icon: Lightning },
  writing: { label: '写作', tone: 'journal', Icon: PencilLine },
  milestone: { label: '里程碑', tone: 'growth', Icon: Flag },
  moment: { label: '瞬间', tone: 'growth', Icon: Camera },
  excerpt: { label: '书摘', tone: 'library', Icon: Quotes },
  note: { label: '笔记', tone: 'library', Icon: BookOpen }
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
  const [completing, setCompleting] = useState<Plan | null>(null);

  const plans = plansQuery.data ?? [];
  const feed = feedQuery.data ?? [];
  const completion = todayCompletion(plans, todayStart);
  const completionRate = completion.total === 0 ? 0 : Math.round((completion.done / completion.total) * 100);
  const panel = todayPlanPanel(plans, todayStart);
  const overdue = panel.filter(
    (plan) => plan.status !== 'done' && plan.scheduledAt != null && plan.scheduledAt < todayStart
  ).length;
  const running =
    tasksQuery.data?.filter((task) => task.status === 'pending' || task.status === 'processing').length ?? 0;
  const error = plansQuery.error ?? feedQuery.error ?? streakQuery.error ?? tasksQuery.error;

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>今天</h1>
        <p className={styles.hint}>{formatDayHeading(todayStart)}</p>
      </header>

      <div className={styles.capture}>
        <PencilSimple className={styles.statIcon} weight="regular" />
        <span className={styles.prompt}>记点什么…文字、里程碑、书摘、计划，都从这里进</span>
        <button
          type="button"
          className={styles.chip}
          data-tone="journal"
          onClick={() => void navigate({ to: PATH.journal })}
        >
          <PencilSimple className={styles.chipIcon} weight="regular" />
          文字
        </button>
        <button
          type="button"
          className={styles.chip}
          data-tone="growth"
          onClick={() => void navigate({ to: PATH.growth })}
        >
          <Flag className={styles.chipIcon} weight="regular" />
          里程碑
        </button>
        <button
          type="button"
          className={styles.chip}
          data-tone="library"
          onClick={() => void navigate({ to: PATH.library })}
        >
          <Quotes className={styles.chipIcon} weight="regular" />
          书摘
        </button>
        <button
          type="button"
          className={styles.chip}
          data-tone="plan"
          onClick={() => void navigate({ to: PATH.plan, search: { create: true } })}
        >
          <ListChecks className={styles.chipIcon} weight="regular" />
          计划
        </button>
      </div>

      {error ? <p className={styles.error}>{errorMessage(error)}</p> : null}

      <div className={styles.stats}>
        <article className={styles.stat}>
          <ListChecks className={styles.statIcon} weight="regular" />
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
          <Notebook className={styles.statIcon} weight="regular" />
          <span className={styles.statValue}>
            {feed.length} <small>条</small>
          </span>
          <span className={styles.statLabel}>今天记录{recordSummary(feed) ? ` · ${recordSummary(feed)}` : ''}</span>
        </article>
        <article className={styles.stat}>
          <Fire className={styles.statIcon} weight="regular" />
          <span className={styles.statValue}>
            {streakCount(streakQuery.data ?? [], todayStart)} <small>天</small>
          </span>
          <span className={styles.statLabel}>连续记录</span>
        </article>
        <article className={styles.stat}>
          <Hourglass className={styles.statIcon} weight="regular" />
          <span className={styles.statValue}>
            {running} <small>运行中</small>
          </span>
          <span className={styles.statLabel}>后台处理</span>
        </article>
      </div>

      <div className={styles.split}>
        <article className={styles.card}>
          <h2 className={styles.cardTitle}>
            <ClockCounterClockwise className={styles.icon} weight="regular" />
            今日动态
            <span className={styles.extra}>按时间混排</span>
          </h2>
          {feedQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}
          {!feedQuery.isPending && feed.length === 0 ? <p className={styles.empty}>今天还没有新的记录。</p> : null}
          {feed.map((item) => (
            <FeedRow key={`${item.kind}-${item.id}`} item={item} />
          ))}
        </article>
        <article className={styles.card}>
          <h2 className={styles.cardTitle}>
            <CalendarBlank className={styles.icon} weight="regular" />
            今日计划
            {overdue > 0 ? <span className={styles.extra}>{overdue} 项逾期</span> : null}
          </h2>
          {plansQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}
          {!plansQuery.isPending && panel.length === 0 ? <p className={styles.empty}>今天还没有计划。</p> : null}
          {panel.map((plan) => (
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
      <CompleteDialog plan={completing} onClose={() => setCompleting(null)} />
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
      {item.locked ? <Lock className={styles.lock} weight="regular" /> : null}
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
