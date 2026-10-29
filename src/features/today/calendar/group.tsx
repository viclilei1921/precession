import { useQuery } from '@tanstack/react-query';
import type { AlmanacHour, CalendarDay } from '@/bridge/calendar';
import { useCardLayout } from '@/features/today/board/board';
import { calendarAlmanacQuery, calendarRangeQuery } from '@/query/calendar/query';
import { timelineListQuery } from '@/query/timeline/query';
import { addLocalDays, isSameLocalDay, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import { AlmanacCard } from './almanac-card';
import { monthGrid, yearGrid, yearRangeHalves } from './bounds';
import styles from './calendar.module.css';
import { HoursCard } from './hours-card';
import { marksByDay } from './marks';
import { MonthGrid } from './month-grid';
import { YearGrid } from './year-grid';

export type CalendarMode = 'month' | 'year';

type CalendarGroupProps = {
  mode: CalendarMode;
  /** 月视图是该月 1 日，年视图是该年 1 月 1 日 */
  cursor: number;
  selected: number;
  columns: number;
  onSelect: (time: number) => void;
  onOpenMonth: (monthStart: number) => void;
};

const noHours: AlmanacHour[] = [];

function dayMap(days: CalendarDay[] | undefined) {
  return new Map((days ?? []).map((day) => [day.date, day]));
}

/** 月历或年历、宜忌、时辰绑在同一列，不拆进瀑布流。 */
export function CalendarGroup({ mode, cursor, selected, columns, onSelect, onOpenMonth }: CalendarGroupProps) {
  const span = mode === 'year' ? yearGrid(cursor) : monthGrid(cursor);
  const halves = mode === 'year' ? yearRangeHalves(cursor) : null;
  const firstFrom = halves ? halves[0].from : toDateInputValue(span.from);
  const firstTo = halves ? halves[0].to : toDateInputValue(span.to);
  const daysQuery = useQuery(calendarRangeQuery(firstFrom, firstTo));
  const restQuery = useQuery({
    ...calendarRangeQuery(halves?.[1].from ?? firstFrom, halves?.[1].to ?? firstTo),
    enabled: halves != null
  });
  const marksQuery = useQuery({
    ...timelineListQuery(span.from, addLocalDays(span.to, 1)),
    enabled: mode === 'month'
  });
  const almanacQuery = useQuery(calendarAlmanacQuery(toDateInputValue(selected)));
  const almanac = almanacQuery.data;

  useCardLayout(
    'calendar',
    [
      mode,
      columns,
      daysQuery.data?.days.length ?? 0,
      restQuery.data?.days.length ?? 0,
      marksQuery.data?.length ?? 0,
      almanac?.yi ?? '',
      almanac?.ji ?? '',
      almanac?.lunar.text ?? '',
      almanac?.hourYiJi.length ?? 0,
      daysQuery.isPending || (halves != null && restQuery.isPending) ? 'pending' : 'ready',
      almanacQuery.isPending ? 'pending' : 'ready'
    ].join(':')
  );

  const days = dayMap([...(daysQuery.data?.days ?? []), ...(restQuery.data?.days ?? [])]);
  const daysError = daysQuery.error ?? restQuery.error;
  const marks = marksByDay(marksQuery.data ?? []);

  return (
    <>
      <article className={styles.card}>
        {daysError ? <p className={styles.error}>{errorMessage(daysError)}</p> : null}
        {mode === 'year' && 'months' in span ? (
          <YearGrid
            months={span.months}
            columns={Math.max(1, columns)}
            days={days}
            selected={selected}
            onSelect={onSelect}
            onOpenMonth={onOpenMonth}
          />
        ) : 'cells' in span ? (
          <MonthGrid cells={span.cells} days={days} marks={marks} selected={selected} onSelect={onSelect} />
        ) : null}
      </article>
      <AlmanacCard
        almanac={almanac}
        pending={almanacQuery.isPending}
        error={almanacQuery.error ? errorMessage(almanacQuery.error) : ''}
      />
      <HoursCard
        hours={almanac?.hourYiJi ?? noHours}
        today={isSameLocalDay(selected, Date.now())}
        pending={almanacQuery.isPending}
        resetKey={toDateInputValue(selected)}
      />
    </>
  );
}
