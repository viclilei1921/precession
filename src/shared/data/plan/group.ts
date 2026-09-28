import type { Plan } from '@/bridge/plan';
import { addLocalDays, formatClock, isSameLocalDay, startOfLocalDay, startOfLocalWeek } from '@/shared/lib/day';

export type PlanGroups = {
  overdue: Plan[];
  today: Plan[];
  tomorrow: Plan[];
  later: Plan[];
};

function bySchedule(left: Plan, right: Plan): number {
  const leftAt = left.scheduledAt ?? left.createdAt;
  const rightAt = right.scheduledAt ?? right.createdAt;
  if (leftAt !== rightAt) {
    return leftAt - rightAt;
  }
  return right.priority - left.priority;
}

function isOpen(plan: Plan): boolean {
  return plan.status === 'scheduled' || plan.status === 'done';
}

/** 待办：未完成的已安排，加上今天完成的 */
export function groupTodo(plans: Plan[], todayStart: number): PlanGroups {
  const tomorrow = addLocalDays(todayStart, 1);
  const afterTomorrow = addLocalDays(todayStart, 2);
  const overdue: Plan[] = [];
  const today: Plan[] = [];
  const tomorrowItems: Plan[] = [];
  const later: Plan[] = [];

  for (const plan of plans) {
    if (plan.status === 'done') {
      if (plan.scheduledAt != null && isSameLocalDay(plan.scheduledAt, todayStart)) {
        today.push(plan);
      }
      continue;
    }
    if (plan.status !== 'scheduled') {
      continue;
    }
    const at = plan.scheduledAt;
    if (at == null || at >= afterTomorrow) {
      later.push(plan);
    } else if (at >= tomorrow) {
      tomorrowItems.push(plan);
    } else if (at >= todayStart) {
      today.push(plan);
    } else {
      overdue.push(plan);
    }
  }

  overdue.sort(bySchedule);
  today.sort(bySchedule);
  tomorrowItems.sort(bySchedule);
  later.sort(bySchedule);
  return { overdue, today, tomorrow: tomorrowItems, later };
}

/** 某一天已安排或已完成的计划 */
export function plansOnDay(plans: Plan[], dayStart: number): Plan[] {
  return plans
    .filter((plan) => isOpen(plan) && plan.scheduledAt != null && isSameLocalDay(plan.scheduledAt, dayStart))
    .sort(bySchedule);
}

/** 今天的已安排（含已完成）和更早仍未完成的 */
export function todayPlanPanel(plans: Plan[], todayStart: number): Plan[] {
  const tomorrow = addLocalDays(todayStart, 1);
  return plans
    .filter((plan) => {
      if (plan.scheduledAt == null || plan.status === 'inbox') {
        return false;
      }
      if (plan.status === 'done') {
        return plan.scheduledAt >= todayStart && plan.scheduledAt < tomorrow;
      }
      return plan.status === 'scheduled' && plan.scheduledAt < tomorrow;
    })
    .sort(bySchedule);
}

/** 今天已安排里，完成数和总数 */
export function todayCompletion(plans: Plan[], todayStart: number): { done: number; total: number } {
  const items = plans.filter(
    (plan) => isOpen(plan) && plan.scheduledAt != null && isSameLocalDay(plan.scheduledAt, todayStart)
  );
  return { done: items.filter((plan) => plan.status === 'done').length, total: items.length };
}

/** 本周已安排里，完成数和总数 */
export function weekCompletion(plans: Plan[], now: number): { done: number; total: number } {
  const start = startOfLocalWeek(now);
  const end = addLocalDays(start, 7);
  const items = plans.filter((plan) => {
    if (!isOpen(plan) || plan.scheduledAt == null) {
      return false;
    }
    return plan.scheduledAt >= start && plan.scheduledAt < end;
  });
  return { done: items.filter((plan) => plan.status === 'done').length, total: items.length };
}

/** 有计划的本地日，给月历标点 */
export function markedDays(plans: Plan[]): Set<number> {
  const days = new Set<number>();
  for (const plan of plans) {
    if (isOpen(plan) && plan.scheduledAt != null) {
      days.add(startOfLocalDay(plan.scheduledAt));
    }
  }
  return days;
}

/** 行尾：已完成看结果，过期看「逾期」，其余看钟点 */
export function planMeta(plan: Plan, todayStart: number): string {
  if (plan.status === 'done') {
    return plan.result.trim() || '已完成';
  }
  if (plan.scheduledAt != null && plan.scheduledAt < todayStart) {
    return '逾期';
  }
  if (plan.scheduledAt != null) {
    return formatClock(plan.scheduledAt);
  }
  return '';
}
