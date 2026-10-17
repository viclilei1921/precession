import { CaretLeftIcon, CaretRightIcon } from '@phosphor-icons/react';
import type { CalendarDay } from '@/bridge/calendar';
import { DayButton } from '@/components/button/day';
import { IconButton } from '@/components/button/icon';
import {
  addLocalDays,
  addLocalMonths,
  formatMonth,
  isSameLocalDay,
  startOfLocalDay,
  toDateInputValue
} from '@/utils/day';
import styles from '../page.module.css';

const WEEKDAYS = ['一', '二', '三', '四', '五', '六', '日'];

type MonthBoardProps = {
  month: number;
  selected: number;
  marked: Set<number>;
  days: CalendarDay[];
  onSelect: (dayStart: number) => void;
  onMonth: (monthStart: number) => void;
};

/** 月历 42 格的起止（含前后补齐），本地当天 0 点 */
export function monthGridBounds(month: number): { from: number; to: number } {
  const first = new Date(month);
  const offset = (first.getDay() + 6) % 7;
  const from = addLocalDays(month, -offset);
  return { from, to: addLocalDays(from, 41) };
}

function dayMark(day: CalendarDay | undefined): string {
  if (!day) {
    return '';
  }
  if (day.holidayName) {
    return day.holidayName;
  }
  if (!day.isWorkday) {
    return '休';
  }
  if (day.isWeekend) {
    return '班';
  }
  return '';
}

export function MonthBoard({ month, selected, marked, days, onSelect, onMonth }: MonthBoardProps) {
  const first = new Date(month);
  const { from } = monthGridBounds(month);
  const today = startOfLocalDay();
  const byDate = new Map(days.map((day) => [day.date, day]));
  const cells = Array.from({ length: 42 }, (_, index) => {
    const time = startOfLocalDay(addLocalDays(from, index));
    return { time, inMonth: new Date(time).getMonth() === first.getMonth() };
  });

  return (
    <div className={styles.month}>
      <div className={styles.monthBar}>
        <IconButton label="上个月" onClick={() => onMonth(addLocalMonths(month, -1))}>
          <CaretLeftIcon weight="regular" />
        </IconButton>
        <span>{formatMonth(month)}</span>
        <IconButton label="下个月" onClick={() => onMonth(addLocalMonths(month, 1))}>
          <CaretRightIcon weight="regular" />
        </IconButton>
      </div>
      <div className={styles.weekdays}>
        {WEEKDAYS.map((label) => (
          <span key={label}>{label}</span>
        ))}
      </div>
      <div className={styles.days}>
        {cells.map((cell) => {
          const selectedDay = isSameLocalDay(cell.time, selected);
          const info = byDate.get(toDateInputValue(cell.time));
          const mark = dayMark(info);
          return (
            <DayButton
              key={cell.time}
              variant="month"
              selected={selectedDay}
              today={isSameLocalDay(cell.time, today)}
              outside={!cell.inMonth}
              off={info != null && !info.isWorkday}
              onClick={() => onSelect(cell.time)}
            >
              {new Date(cell.time).getDate()}
              {mark ? <span className={styles.mark}>{mark}</span> : <span className={styles.markSpacer} />}
              {marked.has(cell.time) ? <i className={styles.dot} /> : <i className={styles.dotSpacer} />}
            </DayButton>
          );
        })}
      </div>
    </div>
  );
}
