import type { Plan } from '@/bridge/plan';
import { formatPlanClock, hasTimeRange, isTimedSingle } from './layout';

// i18n-namespace: plan

type Translate = (key: string, options?: Record<string, unknown>) => string;

/** 色块和日程上的时间文字。全天由语言包决定文案。 */
export function planWhen(plan: Plan, t: Translate): string {
  if (!isTimedSingle(plan) || plan.scheduledAt == null) {
    return t('allDay');
  }
  const start = formatPlanClock(plan.scheduledAt);
  if (!hasTimeRange(plan) || plan.dueAt == null) {
    return start;
  }
  return `${start}–${formatPlanClock(plan.dueAt)}`;
}

/** 月格里的时间：全天计划仍显示开始钟点，钟点计划显示区间。 */
export function planShowTime(plan: Plan, t: Translate): string {
  if (!plan.allDay) {
    return planWhen(plan, t);
  }
  return plan.scheduledAt == null ? '' : formatPlanClock(plan.scheduledAt);
}
