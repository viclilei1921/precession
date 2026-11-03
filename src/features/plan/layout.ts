import type { Plan } from '@/bridge/plan';
import { addLocalDays, addLocalMonths, addLocalYears, startOfLocalDay, startOfLocalWeek } from '@/utils/day';

/** 计划页的五个视图 */
export type PlanView = 'year' | 'month' | 'week' | 'day' | 'agenda';

/** 周一是 0，周日是 6 */
export function weekdayIndex(time: number): number {
  return (new Date(time).getDay() + 6) % 7;
}

/** 周六或周日 */
export function isWeekend(time: number): boolean {
  const day = new Date(time).getDay();
  return day === 0 || day === 6;
}

/** 月格或周格里的一条跨天色块 */
export type BarSegment = {
  plan: Plan;
  col: number;
  span: number;
  lane: number;
  before: boolean;
  after: boolean;
  showTime: boolean;
};

/** 新建抽屉的初始时间 */
export type PlanPreset = {
  date: string;
  end: string;
  allDay: boolean;
  time: string;
  timeEnd: string;
};

/** 清单颜色存的是色相。空值用工作蓝 */
export function groupHue(color: string | undefined): number {
  const hue = Number(color);
  if (color && Number.isFinite(hue)) {
    return ((hue % 360) + 360) % 360;
  }
  return 250;
}

/** 本地日相差的天数 */
export function diffLocalDays(start: number, end: number): number {
  let cursor = startOfLocalDay(start);
  const target = startOfLocalDay(end);
  let count = 0;
  while (cursor < target) {
    cursor = addLocalDays(cursor, 1);
    count += 1;
  }
  while (cursor > target) {
    cursor = addLocalDays(cursor, -1);
    count -= 1;
  }
  return count;
}

/** 计划开始那一天。没有安排时间时为空 */
export function planStart(plan: Plan): number | null {
  return plan.scheduledAt == null ? null : startOfLocalDay(plan.scheduledAt);
}

/** 计划结束那一天，含当天。没有截止时等于开始 */
export function planFinish(plan: Plan): number | null {
  const start = planStart(plan);
  if (start == null) {
    return null;
  }
  if (plan.dueAt == null) {
    return start;
  }
  const end = startOfLocalDay(plan.dueAt);
  return Math.max(end, start);
}

/** 同一天里的非全天计划 */
export function isTimedSingle(plan: Plan): boolean {
  if (plan.allDay || plan.scheduledAt == null) {
    return false;
  }
  const end = plan.dueAt == null ? plan.scheduledAt : plan.dueAt;
  return startOfLocalDay(plan.scheduledAt) === startOfLocalDay(end);
}

/** 同一天里写出了结束时间 */
export function hasTimeRange(plan: Plan): boolean {
  return isTimedSingle(plan) && plan.dueAt != null && plan.scheduledAt != null && plan.dueAt > plan.scheduledAt;
}

/** 8:00 这种钟点 */
export function formatPlanClock(time: number): string {
  const date = new Date(time);
  return `${date.getHours()}:${String(date.getMinutes()).padStart(2, '0')}`;
}

/** 这一天有交集的计划，含跨天 */
export function plansOnDay(plans: Plan[], day: number): Plan[] {
  const key = startOfLocalDay(day);
  return plans.filter((plan) => {
    const start = planStart(plan);
    const finish = planFinish(plan);
    return start != null && finish != null && key >= start && key <= finish;
  });
}

/** 从周一开始的六周 */
export function monthWeeks(monthStart: number): number[][] {
  const monday = startOfLocalWeek(monthStart);
  return Array.from({ length: 6 }, (_, week) =>
    Array.from({ length: 7 }, (_, index) => addLocalDays(monday, week * 7 + index))
  );
}

/** 年历里的一个月。第六周整周都在月外时不再画出 */
export function miniDays(monthStart: number): Array<{ day: number; outside: boolean }> {
  const monday = startOfLocalWeek(monthStart);
  const month = new Date(monthStart).getMonth();
  const days: Array<{ day: number; outside: boolean }> = [];
  for (let index = 0; index < 42; index += 1) {
    const day = addLocalDays(monday, index);
    if (index === 35 && new Date(day).getMonth() !== month) {
      break;
    }
    days.push({ day, outside: new Date(day).getMonth() !== month });
  }
  return days;
}

/** 把计划排进这一周的泳道。周和日的全天栏不含同一天的钟点计划 */
export function layoutBars(days: number[], plans: Plan[], includeTimed = false): BarSegment[] {
  const weekStart = startOfLocalDay(days[0] ?? 0);
  const weekEnd = startOfLocalDay(days[days.length - 1] ?? weekStart);
  const segments: Array<Omit<BarSegment, 'lane'>> = [];
  for (const plan of plans) {
    if (!includeTimed && isTimedSingle(plan)) {
      continue;
    }
    const start = planStart(plan);
    const finish = planFinish(plan);
    if (start == null || finish == null || finish < weekStart || start > weekEnd) {
      continue;
    }
    const segStart = Math.max(start, weekStart);
    const segEnd = Math.min(finish, weekEnd);
    segments.push({
      plan,
      col: diffLocalDays(weekStart, segStart),
      span: diffLocalDays(segStart, segEnd) + 1,
      before: start < weekStart,
      after: finish > weekEnd,
      showTime: !plan.allDay && plan.scheduledAt != null && segEnd === finish
    });
  }
  segments.sort((left, right) => {
    if (left.col !== right.col) {
      return left.col - right.col;
    }
    if (left.span !== right.span) {
      return right.span - left.span;
    }
    const leftTime = left.showTime && left.plan.scheduledAt != null ? left.plan.scheduledAt : 0;
    const rightTime = right.showTime && right.plan.scheduledAt != null ? right.plan.scheduledAt : 0;
    if (leftTime !== rightTime) {
      return leftTime - rightTime;
    }
    return left.plan.title.localeCompare(right.plan.title, 'zh');
  });
  const lanes: Array<Array<{ col: number; span: number }>> = [];
  return segments.map((segment) => {
    let lane = lanes.findIndex((items) =>
      items.every((other) => segment.col >= other.col + other.span || other.col >= segment.col + segment.span)
    );
    if (lane < 0) {
      lane = lanes.length;
      lanes.push([]);
    }
    lanes[lane]?.push({ col: segment.col, span: segment.span });
    return { ...segment, lane };
  });
}

/** 按视图移动光标 */
export function shiftCursor(view: PlanView, cursor: number, dir: number): number {
  if (view === 'day') {
    return addLocalDays(cursor, dir);
  }
  if (view === 'week') {
    return addLocalDays(cursor, dir * 7);
  }
  if (view === 'year') {
    return addLocalYears(cursor, dir);
  }
  return addLocalMonths(cursor, dir);
}

/** 分钟数写成 09:00 */
export function clockLabel(minutes: number): string {
  const hour = Math.floor(minutes / 60);
  const minute = minutes % 60;
  return `${String(hour).padStart(2, '0')}:${String(minute).padStart(2, '0')}`;
}

/** 格子上松开后的开始和结束。结束为空表示这一下没有截止 */
export function gridPlacement(
  day: number,
  start: number,
  end: number | null
): { scheduledAt: number; dueAt: number | null } {
  const scheduledAt = atClock(day, clockLabel(start));
  if (end == null) {
    return { scheduledAt, dueAt: null };
  }
  const dueAt = end >= 24 * 60 ? atClock(day, '23:59') : atClock(day, clockLabel(end));
  return { scheduledAt, dueAt: dueAt > scheduledAt ? dueAt : null };
}

/** 把日期和钟点合成时间戳 */
export function atClock(day: number, clock: string): number {
  const [hour, minute] = clock.split(':').map(Number);
  const date = new Date(startOfLocalDay(day));
  date.setHours(hour || 0, minute || 0, 0, 0);
  return date.getTime();
}

/** 当月最后一天的 0 点 */
export function endOfLocalMonth(time: number): number {
  const date = new Date(time);
  return new Date(date.getFullYear(), date.getMonth() + 1, 0).getTime();
}
