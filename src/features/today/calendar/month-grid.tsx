import { useTranslation } from 'react-i18next';
import type { CalendarDay } from '@/bridge/calendar';
import { isSameLocalDay, toDateInputValue } from '@/utils/day';
import type { DayCell } from './bounds';
import styles from './calendar.module.css';
import { dayFace } from './day-face';
import type { MarkTone } from './marks';

type MonthGridProps = {
  cells: DayCell[];
  days: Map<string, CalendarDay>;
  marks: Map<string, MarkTone[]>;
  selected: number;
  onSelect: (time: number) => void;
};

const WEEKDAY_KEYS = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'] as const;

export function MonthGrid({ cells, days, marks, selected, onSelect }: MonthGridProps) {
  const { t } = useTranslation('today');
  const today = Date.now();

  const LEGEND: { tone: MarkTone; label: string }[] = [
    { tone: 'plan', label: t('calendar.legend.plan') },
    { tone: 'journal', label: t('calendar.legend.journal') },
    { tone: 'growth', label: t('calendar.legend.growth') },
    { tone: 'library', label: t('calendar.legend.library') }
  ];

  return (
    <>
      <div className={styles.weekdays}>
        {WEEKDAY_KEYS.map((key) => (
          <span key={key}>{t(`weekdayShort.${key}`)}</span>
        ))}
      </div>
      <div className={styles.days}>
        {cells.map((cell) => {
          const info = days.get(toDateInputValue(cell.time));
          const face = dayFace(info);
          const tones = marks.get(toDateInputValue(cell.time)) ?? [];
          const date = new Date(cell.time);
          const label = t('calendar.monthCellLabel', {
            month: date.getMonth() + 1,
            day: date.getDate(),
            lunarText: face.text
          });
          return (
            <button
              key={cell.time}
              type="button"
              className={styles.day}
              data-out={cell.inMonth ? undefined : 'true'}
              data-weekend={date.getDay() === 0 || date.getDay() === 6 ? 'true' : undefined}
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
