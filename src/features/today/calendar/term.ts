import type { CalendarDay } from '@/bridge/calendar';
import { fromDateInputValue, toDateInputValue } from '@/utils/day';

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
    .map((day) => ({ date: day.date, name: day.lunar?.solarTerm ?? '' }))
    .sort((left, right) => left.date.localeCompare(right.date));

  let currentIndex = -1;
  for (let index = 0; index < terms.length; index += 1) {
    if (terms[index].date <= selectedKey) {
      currentIndex = index;
    }
  }
  const current = currentIndex >= 0 ? terms[currentIndex] : undefined;
  const next = currentIndex >= 0 ? terms[currentIndex + 1] : terms.find((item) => item.date > selectedKey);
  const name = current?.name || fallbackName || '节气';
  if (!current || !next) {
    const hint = next ? `距${next.name}还有 ${daysBetween(selectedKey, next.date)} 天` : '';
    return { name, hint, percent: 0 };
  }
  const span = Math.max(1, daysBetween(current.date, next.date));
  const elapsed = Math.min(span, Math.max(0, daysBetween(current.date, selectedKey)));
  const hint = elapsed === 0 ? `当日交节 · 距${next.name}还有 ${span} 天` : `距${next.name}还有 ${span - elapsed} 天`;
  return { name, hint, percent: Math.round((elapsed / span) * 100) };
}
