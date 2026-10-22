import type { CalendarDay } from '@/bridge/calendar';
import { isSameLocalDay, toDateInputValue } from '@/utils/day';
import type { DayCell } from './bounds';
import styles from './calendar.module.css';
import { dayFace } from './day-face';
import type { MarkTone } from './marks';

const WEEKDAYS = ['一', '二', '三', '四', '五', '六', '日'];
const LEGEND: { tone: MarkTone; label: string }[] = [
  { tone: 'plan', label: '计划' },
  { tone: 'journal', label: '手记' },
  { tone: 'growth', label: '成长' },
  { tone: 'library', label: '书库' }
];

type MonthGridProps = {
  cells: DayCell[];
  days: Map<string, CalendarDay>;
  marks: Map<string, MarkTone[]>;
  selected: number;
  onSelect: (time: number) => void;
};

export function MonthGrid({ cells, days, marks, selected, onSelect }: MonthGridProps) {
  const today = Date.now();

  return (
    <>
      <div className={styles.weekdays}>
        {WEEKDAYS.map((label) => (
          <span key={label}>{label}</span>
        ))}
      </div>
      <div className={styles.days}>
        {cells.map((cell) => {
          const info = days.get(toDateInputValue(cell.time));
          const face = dayFace(info);
          const tones = marks.get(toDateInputValue(cell.time)) ?? [];
          const date = new Date(cell.time);
          const label = `${date.getMonth() + 1}月${date.getDate()}日 ${face.text}`;
          return (
            <button
              key={cell.time}
              type="button"
              className={styles.day}
              data-out={cell.inMonth ? undefined : 'true'}
              data-today={isSameLocalDay(cell.time, today) ? 'true' : undefined}
              data-selected={isSameLocalDay(cell.time, selected) ? 'true' : undefined}
              aria-label={label}
              onClick={() => onSelect(cell.time)}
            >
              <span className={styles.num}>{date.getDate()}</span>
              <span className={styles.sub} data-kind={face.kind}>
                {face.text}
              </span>
              <span className={styles.dots}>
                {tones.map((tone) => (
                  <i key={tone} className={styles.dot} data-tone={tone} />
                ))}
              </span>
            </button>
          );
        })}
      </div>
      <p className={styles.legend}>
        {LEGEND.map((item) => (
          <span key={item.tone}>
            <i className={styles.dot} data-tone={item.tone} />
            {item.label}
          </span>
        ))}
      </p>
    </>
  );
}
