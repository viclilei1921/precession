import { useTranslation } from 'react-i18next';
import type { Plan } from '@/bridge/plan';
import { Scroll } from '@/components/scroll';
import { formatDate, weekdayShort } from '@/utils/day';
import styles from '../calendar.module.css';
import { planWhen } from '../format';
import { endOfLocalMonth, planFinish, planStart } from '../layout';
import { PlanChip } from './plan-chip';

type AgendaViewProps = {
  through: number;
  plans: Plan[];
  hueOf: (plan: Plan) => number;
  onOpen: (plan: Plan) => void;
  onToggle: (plan: Plan) => void;
  onMore: () => void;
};

/** 按月份列出计划。没有日期的放在最上面，这里不能新建 */
export function AgendaView({ through, plans, hueOf, onOpen, onToggle, onMore }: AgendaViewProps) {
  const { t } = useTranslation('plan');
  const limit = endOfLocalMonth(through);
  const undated = plans.filter((plan) => plan.scheduledAt == null);
  const dated = plans.filter((plan) => {
    const start = planStart(plan);
    return start != null && start <= limit;
  });
  const groups = new Map<string, Plan[]>();
  for (const plan of dated) {
    const start = new Date(plan.scheduledAt ?? 0);
    const key = `${start.getFullYear()}-${start.getMonth()}`;
    const items = groups.get(key) ?? [];
    items.push(plan);
    groups.set(key, items);
  }
  const keys = [...groups.keys()].sort();

  return (
    <Scroll className={styles.fill} axis="vertical">
      {undated.length > 0 ? (
        <section>
          <h2 className={styles.agMonth}>{t('undated')}</h2>
          {undated.map((plan) => (
            <AgendaRow key={plan.id} plan={plan} hue={hueOf(plan)} onOpen={onOpen} onToggle={onToggle} />
          ))}
        </section>
      ) : null}
      {keys.length === 0 && undated.length === 0 ? <p className={styles.empty}>{t('empty')}</p> : null}
      {keys.map((key) => {
        const items = groups.get(key) ?? [];
        const [year, month] = key.split('-').map(Number);
        const byDay = new Map<number, Plan[]>();
        for (const plan of items.sort(byAgenda)) {
          const start = planStart(plan);
          if (start == null) {
            continue;
          }
          const dayItems = byDay.get(start) ?? [];
          dayItems.push(plan);
          byDay.set(start, dayItems);
        }
        return (
          <section key={key} id={`plan-month-${year}-${month}`}>
            <h2 className={styles.agMonth}>{t('agenda.monthLabel', { year, month: (month ?? 0) + 1 })}</h2>
            {[...byDay.keys()].sort().map((day) => (
              <div key={day} className={styles.agDay}>
                <div className={styles.agDate}>
                  <b>{new Date(day).getDate()}</b>
                  <span>{weekdayShort(day)}</span>
                </div>
                <div>
                  {(byDay.get(day) ?? []).map((plan) => (
                    <AgendaRow key={plan.id} plan={plan} hue={hueOf(plan)} onOpen={onOpen} onToggle={onToggle} />
                  ))}
                </div>
              </div>
            ))}
          </section>
        );
      })}
      <div className={styles.footer}>
        <span>{t('showingUntil', { date: formatDate(limit) })}</span>
        <button type="button" onClick={onMore}>
          {t('showMore')}
        </button>
      </div>
    </Scroll>
  );
}

function AgendaRow({
  plan,
  hue,
  onOpen,
  onToggle
}: {
  plan: Plan;
  hue: number;
  onOpen: (plan: Plan) => void;
  onToggle: (plan: Plan) => void;
}) {
  const { t } = useTranslation('plan');
  const range = rangeText(plan);
  return (
    <div className={styles.agRow}>
      <span className={styles.agWhen}>{plan.scheduledAt == null ? '' : planWhen(plan, t)}</span>
      <PlanChip plan={plan} hue={hue} detail={range} className={styles.agChip} onOpen={onOpen} onToggle={onToggle} />
    </div>
  );
}

function rangeText(plan: Plan): string {
  const start = planStart(plan);
  const finish = planFinish(plan);
  if (start == null || finish == null || finish === start) {
    return '';
  }
  return `${formatDate(start)} – ${formatDate(finish)}`;
}

function byAgenda(left: Plan, right: Plan): number {
  const leftStart = left.scheduledAt ?? 0;
  const rightStart = right.scheduledAt ?? 0;
  if (leftStart !== rightStart) {
    return leftStart - rightStart;
  }
  return left.title.localeCompare(right.title);
}
