import type { CalendarDay } from '@/bridge/calendar';
import i18n from '@/i18n';
import { fromDateInputValue, toDateInputValue } from '@/utils/day';
import { translateSolarTerm } from './almanac-dict';

const DAY = 24 * 60 * 60 * 1000;

export type TermView = {
  name: string;
  hint: string;
  percent: number;
};

function daysBetween(from: string, to: string): number {
  const start = fromDateInputValue(from);
  const end = fromDateInputValue(to);
  if (start == null || end == null) {
    return 0;
  }
  return Math.round((end - start) / DAY);
}

/**
 * 当前节气和距下一节气的进度。
 *
 * 节气日取日历格子上的 `lunar.solarTerm`。没有下一节气时只显示名称。
 */
export function termView(days: CalendarDay[], selected: number, fallbackName: string): TermView {
  const selectedKey = toDateInputValue(selected);
  const terms = days
    .filter((day) => day.lunar?.solarTerm)
    .map((day) => ({ date: day.date, name: translateSolarTerm(day.lunar?.solarTerm ?? '') }))
    .sort((left, right) => left.date.localeCompare(right.date));

  let currentIndex = -1;
  for (let index = 0; index < terms.length; index += 1) {
    if (terms[index].date <= selectedKey) {
      currentIndex = index;
    }
  }
  const current = currentIndex >= 0 ? terms[currentIndex] : undefined;
  const next = currentIndex >= 0 ? terms[currentIndex + 1] : terms.find((item) => item.date > selectedKey);
  const name = current?.name || translateSolarTerm(fallbackName) || i18n.t('today:calendar.term.fallback');
  if (!current || !next) {
    const hint = next
      ? i18n.t('today:calendar.term.hint', { count: daysBetween(selectedKey, next.date), name: next.name })
      : '';
    return { name, hint, percent: 0 };
  }
  const span = Math.max(1, daysBetween(current.date, next.date));
  const elapsed = Math.min(span, Math.max(0, daysBetween(current.date, selectedKey)));
  const hint =
    elapsed === 0
      ? i18n.t('today:calendar.term.hintToday', { count: span, name: next.name })
      : i18n.t('today:calendar.term.hint', { count: span - elapsed, name: next.name });
  return { name, hint, percent: Math.round((elapsed / span) * 100) };
}
