import { queryOptions } from '@tanstack/react-query';
import type { CalendarDay } from '@/bridge/calendar';
import { calendarList } from '@/bridge/calendar';

/** 今天和计划月历共用这一段日期 */
export function calendarRangeQuery(from: string, to: string) {
  return queryOptions({
    queryKey: ['calendar', 'range', from, to],
    queryFn: () => calendarList(from, to)
  });
}

/** 农历、节日，以及上班或休息 */
export function calendarCaption(day: CalendarDay | undefined): string {
  if (!day) {
    return '';
  }
  const parts: string[] = [];
  if (day.lunar?.text) {
    parts.push(day.lunar.text);
  }
  if (day.holidayName) {
    parts.push(day.holidayName);
  }
  parts.push(day.isWorkday ? '上班' : '休息');
  return parts.join(' · ');
}
