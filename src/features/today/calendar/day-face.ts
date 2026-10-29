import type { CalendarDay, Lunar } from '@/bridge/calendar';

export type DayKind = 'holiday' | 'term' | 'lunar';

export type DayFace = {
  text: string;
  kind: DayKind;
};

/** 初一显示月份，其余显示日名。农历全文形如「九月十三」。 */
export function lunarDayLabel(lunar: Lunar | null | undefined): string {
  if (!lunar?.text) {
    return '';
  }
  if (lunar.day === 1) {
    return lunar.text.slice(0, -2);
  }
  return lunar.text.slice(-2);
}

/** 格子副文：节日优先，其次节气，再是农历。 */
export function dayFace(day: CalendarDay | undefined): DayFace {
  if (!day) {
    return { text: '', kind: 'lunar' };
  }
  if (day.holidayName) {
    return { text: day.holidayName, kind: 'holiday' };
  }
  if (day.lunar?.solarTerm) {
    return { text: day.lunar.solarTerm, kind: 'term' };
  }
  return { text: lunarDayLabel(day.lunar), kind: 'lunar' };
}

export type YearFace = DayFace & {
  badge: '' | '休' | '班';
  rest: boolean;
};

/** 年历格子：不上班为休，周末仍上班为班。 */
export function yearFace(day: CalendarDay | undefined): YearFace {
  const face = dayFace(day);
  if (!day) {
    return { ...face, badge: '', rest: false };
  }
  const makeup = day.isWeekend && day.isWorkday;
  const rest = !day.isWorkday;
  return { ...face, badge: makeup ? '班' : rest ? '休' : '', rest };
}
