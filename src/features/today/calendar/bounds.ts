import { addLocalDays, startOfLocalDay, toDateInputValue } from '@/utils/day';

export type DayCell = {
  time: number;
  inMonth: boolean;
};

/** 月历 42 格，周一开始，含前后补齐 */
export function monthGrid(monthStart: number): { from: number; to: number; cells: DayCell[] } {
  const first = new Date(monthStart);
  const offset = (first.getDay() + 6) % 7;
  const from = addLocalDays(monthStart, -offset);
  const cells = Array.from({ length: 42 }, (_, index) => {
    const time = startOfLocalDay(addLocalDays(from, index));
    return {
      time,
      inMonth: new Date(time).getMonth() === first.getMonth() && new Date(time).getFullYear() === first.getFullYear()
    };
  });
  return { from, to: cells[41].time, cells };
}

export type YearMonth = {
  monthStart: number;
  label: string;
  cells: DayCell[];
};

/** 一年十二个月的格子。起止含一月之前和十二月之后的补齐天。 */
export function yearGrid(yearStart: number): { from: number; to: number; months: YearMonth[] } {
  const year = new Date(yearStart).getFullYear();
  const months = Array.from({ length: 12 }, (_, month) => {
    const monthStart = new Date(year, month, 1).getTime();
    const grid = monthGrid(monthStart);
    return { monthStart, label: `${month + 1}月`, cells: grid.cells };
  });
  const last = months[11].cells;
  return { from: months[0].cells[0].time, to: last[last.length - 1].time, months };
}

/**
 * 年历含前后补齐，整段会超过接口允许的 366 天。
 * 从七月切开，两段都盖住补齐天。
 */
export function yearRangeHalves(yearStart: number): { from: string; to: string }[] {
  const grid = yearGrid(yearStart);
  const year = new Date(yearStart).getFullYear();
  const july = new Date(year, 6, 1).getTime();
  return [
    { from: toDateInputValue(grid.from), to: toDateInputValue(addLocalDays(july, -1)) },
    { from: toDateInputValue(july), to: toDateInputValue(grid.to) }
  ];
}
