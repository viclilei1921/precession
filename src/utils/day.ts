import i18n, { currentLocale } from '@/i18n';

function dateFormatter(options: Intl.DateTimeFormatOptions): Intl.DateTimeFormat {
  return new Intl.DateTimeFormat(currentLocale(), options);
}

/** 本地当天 0 点的毫秒时间戳 */
export function startOfLocalDay(time = Date.now()): number {
  const date = new Date(time);
  date.setHours(0, 0, 0, 0);
  return date.getTime();
}

/** 本地当月 1 日 0 点 */
export function startOfLocalMonth(time = Date.now()): number {
  const date = new Date(time);
  return new Date(date.getFullYear(), date.getMonth(), 1).getTime();
}

/** 本地当年 1 月 1 日 0 点 */
export function startOfLocalYear(time = Date.now()): number {
  return new Date(new Date(time).getFullYear(), 0, 1).getTime();
}

/** 加减年份，落到该年 1 月 1 日 */
export function addLocalYears(time: number, years: number): number {
  const date = new Date(time);
  return new Date(date.getFullYear() + years, 0, 1).getTime();
}

/** 生日到某一天的年龄，拆成可翻译的数字。不满两岁用月龄 */
export type AgeParts = { months: number; years: number; rest: number };

export function ageParts(birthday: number, at = Date.now()): AgeParts {
  const birth = new Date(birthday);
  const when = new Date(at);
  let months = (when.getFullYear() - birth.getFullYear()) * 12 + (when.getMonth() - birth.getMonth());
  if (when.getDate() < birth.getDate()) {
    months -= 1;
  }
  if (months < 0) {
    return { months: -1, years: 0, rest: 0 };
  }
  return { months, years: Math.floor(months / 12), rest: months % 12 };
}

/** 生日到某一天的年龄。不满两岁用月龄 */
export function ageLabel(birthday: number, at = Date.now()): string {
  const { months, years, rest } = ageParts(birthday, at);
  if (months < 0) {
    return '';
  }
  if (months === 0) {
    return i18n.t('common:age.birthDay');
  }
  if (months < 24) {
    return i18n.t('common:age.months', { count: months });
  }
  if (rest === 0) {
    return i18n.t('common:age.years', { count: years });
  }
  return i18n.t('common:age.yearsMonths', { years, months: rest });
}

/** 本地周一 0 点 */
export function startOfLocalWeek(time = Date.now()): number {
  const start = startOfLocalDay(time);
  const day = new Date(start).getDay();
  const offset = day === 0 ? 6 : day - 1;
  return addLocalDays(start, -offset);
}

/** 在本地日历上加减天数 */
export function addLocalDays(time: number, days: number): number {
  const date = new Date(time);
  date.setDate(date.getDate() + days);
  return date.getTime();
}

/** 在本地日历上加减月份，落到该月 1 日 */
export function addLocalMonths(time: number, months: number): number {
  const date = new Date(time);
  return new Date(date.getFullYear(), date.getMonth() + months, 1).getTime();
}

/** 两个时间戳是否在同一个本地日 */
export function isSameLocalDay(left: number, right: number): boolean {
  return startOfLocalDay(left) === startOfLocalDay(right);
}

/** `YYYY-MM-DD`，给日期输入框用 */
export function toDateInputValue(time: number | null): string {
  if (time == null) {
    return '';
  }
  const date = new Date(time);
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${date.getFullYear()}-${month}-${day}`;
}

/** 日期输入框的值转成本地当天 0 点。空字符串得到 null */
export function fromDateInputValue(value: string): number | null {
  if (!value) {
    return null;
  }
  const [year, month, day] = value.split('-').map(Number);
  if (!year || !month || !day) {
    return null;
  }
  return new Date(year, month - 1, day).getTime();
}

/** 9月28日 / Nov 3 */
export function formatMonthDay(time: number): string {
  return dateFormatter({ month: 'short', day: 'numeric' }).format(new Date(time));
}

/** 9月28日 周一 / Nov 3, Mon */
export function formatDayHeading(time: number): string {
  return `${formatMonthDay(time)} ${weekdayShort(time)}`;
}

/** 9月 / Nov */
export function formatMonth(time: number): string {
  return dateFormatter({ month: 'short' }).format(new Date(time));
}

/** 2026年11月3日 / Nov 3, 2026 */
export function formatDate(time: number): string {
  return dateFormatter({ year: 'numeric', month: 'short', day: 'numeric' }).format(new Date(time));
}

/** 2026年11月 / November 2026 */
export function formatYearMonth(time: number): string {
  return dateFormatter({ year: 'numeric', month: 'long' }).format(new Date(time));
}

/** 2026年 / 2026 */
export function formatYear(time: number): string {
  return dateFormatter({ year: 'numeric' }).format(new Date(time));
}

/** 周一 / Mon */
export function weekdayShort(time: number): string {
  return dateFormatter({ weekday: 'short' }).format(new Date(time));
}

/** 一 / M */
export function weekdayNarrow(time: number): string {
  return dateFormatter({ weekday: 'narrow' }).format(new Date(time));
}

/** 当天 0 点得到空字符串，其余得到 HH:mm */
export function formatClock(time: number): string {
  const date = new Date(time);
  const hours = date.getHours();
  const minutes = date.getMinutes();
  if (hours === 0 && minutes === 0) {
    return '';
  }
  return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`;
}

/** 时间轴上的钟点，0 点也显示 */
export function formatFeedClock(time: number): string {
  const date = new Date(time);
  return `${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`;
}
