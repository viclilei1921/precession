import { CaretLeft, CaretRight } from '@phosphor-icons/react';
import { addLocalDays, addLocalMonths, formatMonth, isSameLocalDay, startOfLocalDay } from '@/shared/lib/day';
import styles from '../page.module.css';

const WEEKDAYS = ['一', '二', '三', '四', '五', '六', '日'];

type MonthBoardProps = {
  month: number;
  selected: number;
  marked: Set<number>;
  onSelect: (dayStart: number) => void;
  onMonth: (monthStart: number) => void;
};

export function MonthBoard({ month, selected, marked, onSelect, onMonth }: MonthBoardProps) {
  const first = new Date(month);
  const offset = (first.getDay() + 6) % 7;
  const gridStart = addLocalDays(month, -offset);
  const today = startOfLocalDay();
  const cells = Array.from({ length: 42 }, (_, index) => {
    const time = startOfLocalDay(addLocalDays(gridStart, index));
    return { time, inMonth: new Date(time).getMonth() === first.getMonth() };
  });

  return (
    <div className={styles.month}>
      <div className={styles.monthBar}>
        <button
          type="button"
          className={styles.iconButton}
          aria-label="上个月"
          onClick={() => onMonth(addLocalMonths(month, -1))}
        >
          <CaretLeft className={styles.icon} weight="regular" />
        </button>
        <span>{formatMonth(month)}</span>
        <button
          type="button"
          className={styles.iconButton}
          aria-label="下个月"
          onClick={() => onMonth(addLocalMonths(month, 1))}
        >
          <CaretRight className={styles.icon} weight="regular" />
        </button>
      </div>
      <div className={styles.weekdays}>
        {WEEKDAYS.map((label) => (
          <span key={label}>{label}</span>
        ))}
      </div>
      <div className={styles.days}>
        {cells.map((cell) => {
          const selectedDay = isSameLocalDay(cell.time, selected);
          return (
            <button
              key={cell.time}
              type="button"
              className={styles.day}
              data-selected={selectedDay ? 'true' : undefined}
              data-today={isSameLocalDay(cell.time, today) ? 'true' : undefined}
              data-muted={cell.inMonth ? undefined : 'true'}
              onClick={() => onSelect(cell.time)}
            >
              {new Date(cell.time).getDate()}
              {marked.has(cell.time) ? <i className={styles.dot} /> : <i className={styles.dotSpacer} />}
            </button>
          );
        })}
      </div>
    </div>
  );
}
