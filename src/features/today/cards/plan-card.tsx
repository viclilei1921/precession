import { ListChecksIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useTranslation } from 'react-i18next';
import { useCardLayout } from '@/features/today/board/board';
import { todayCompletion, todayPlanPanel } from '@/query/plan/group';
import { planListQuery } from '@/query/plan/query';
import { PATH } from '@/router/path';
import { startOfLocalDay } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import styles from './tile.module.css';
import { TileHead } from './tile-head';

export function PlanCard() {
  const { t } = useTranslation('today');
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
        title={t('card.plan')}
        extra={overdue > 0 ? t('planCard.overdue', { count: overdue }) : undefined}
        warn={overdue > 0}
      />
      {plansQuery.error ? <p className={styles.error}>{errorMessage(plansQuery.error)}</p> : null}
      <p className={styles.figure}>
        {completion.done}
        <small> / {completion.total}</small>
      </p>
      <div className={styles.bar} role="img" aria-label={t('planCard.complete', { percent })}>
        <span style={{ width: `${percent}%` }} />
      </div>
      <p className={styles.next}>{next ? t('planCard.next', { title: next.title }) : t('planCard.noNext')}</p>
    </button>
  );
}
