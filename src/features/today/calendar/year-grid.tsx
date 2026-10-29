import type { CalendarDay } from '@/bridge/calendar';
import { isSameLocalDay, toDateInputValue } from '@/utils/day';
import type { YearMonth } from './bounds';
import styles from './calendar.module.css';
import { yearFace } from './day-face';

const WEEKDAYS = ['一', '二', '三', '四', '五', '六', '日'];

type YearGridProps = {
  months: YearMonth[];
  columns: number;
  days: Map<string, CalendarDay>;
  selected: number;
  onSelect: (time: number) => void;
  onOpenMonth: (monthStart: number) => void;
};

export function YearGrid({ months, columns, days, selected, onSelect, onOpenMonth }: YearGridProps) {
  const today = Date.now();

  return (
    <div className={styles.yearGrid} style={{ gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))` }}>
      {months.map((month) => (
        <section key={month.monthStart} className={styles.mini}>
          <button type="button" className={styles.miniName} onClick={() => onOpenMonth(month.monthStart)}>
            {month.label}
          </button>
          <div className={styles.weekdays}>
            {WEEKDAYS.map((label) => (
              <span key={label}>{label}</span>
            ))}
          </div>
          <div className={styles.miniDays}>
            {month.cells.map((cell) => {
              const info = days.get(toDateInputValue(cell.time));
              const face = yearFace(info);
              const date = new Date(cell.time);
              const label = `${date.getMonth() + 1}月${date.getDate()}日 ${face.text} ${face.badge}`.trim();
              return (
                <button
                  key={cell.time}
                  type="button"
                  className={styles.yday}
                  data-out={cell.inMonth ? undefined : 'true'}
                  data-rest={face.rest ? 'true' : undefined}
                  data-today={isSameLocalDay(cell.time, today) ? 'true' : undefined}
                  data-selected={isSameLocalDay(cell.time, selected) ? 'true' : undefined}
                  aria-label={label}
                  onClick={() => onSelect(cell.time)}
                >
                  <span className={styles.ydayNum}>{date.getDate()}</span>
                  <span className={styles.ydayBadge}>{face.badge}</span>
                  <span className={styles.ydaySub} data-kind={face.kind}>
                    {face.text}
                  </span>
                </button>
              );
            })}
          </div>
        </section>
      ))}
    </div>
  );
}
