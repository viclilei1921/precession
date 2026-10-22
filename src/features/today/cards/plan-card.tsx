import { ListChecksIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useCardLayout } from '@/features/today/board/board';
import { todayCompletion, todayPlanPanel } from '@/query/plan/group';
import { planListQuery } from '@/query/plan/query';
import { PATH } from '@/router/path';
import { startOfLocalDay } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import styles from './tile.module.css';
import { TileHead } from './tile-head';

export function PlanCard() {
  const navigate = useNavigate();
  const plansQuery = useQuery(planListQuery);
  const today = startOfLocalDay();
  const plans = plansQuery.data ?? [];
  const panel = todayPlanPanel(plans, today);
  const completion = todayCompletion(plans, today);
  const overdue = panel.filter(
    (plan) => plan.status !== 'done' && plan.scheduledAt != null && plan.scheduledAt < today
  ).length;
  const next = panel.find((plan) => plan.status !== 'done');
  const percent = completion.total === 0 ? 0 : Math.round((completion.done / completion.total) * 100);

  useCardLayout(
    'plan',
    `${plansQuery.isPending}:${completion.done}:${completion.total}:${overdue}:${next?.title ?? ''}`
  );

  return (
    <button type="button" className={styles.card} onClick={() => void navigate({ to: PATH.plan })}>
      <TileHead
        tone="plan"
        icon={ListChecksIcon}
        title="计划"
        extra={overdue > 0 ? `${overdue} 项逾期` : undefined}
        warn={overdue > 0}
      />
      {plansQuery.error ? <p className={styles.error}>{errorMessage(plansQuery.error)}</p> : null}
      <p className={styles.figure}>
        {completion.done}
        <small> / {completion.total}</small>
      </p>
      <div className={styles.bar} role="img" aria-label={`完成 ${percent}%`}>
        <span style={{ width: `${percent}%` }} />
      </div>
      <p className={styles.next}>{next ? `下一件 · ${next.title}` : '今天没有未完成的计划'}</p>
    </button>
  );
}
