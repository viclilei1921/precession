import { CaretLeftIcon, CaretRightIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { IconButton } from '@/components/button/icon';
import { Scroll } from '@/components/scroll';
import { calendarRangeQuery } from '@/query/calendar/query';
import {
  addLocalMonths,
  addLocalYears,
  formatMonthDay,
  startOfLocalDay,
  startOfLocalMonth,
  startOfLocalYear,
  toDateInputValue
} from '@/utils/day';
import { Board } from './board/board';
import { CAL_MIN, YEAR_MIN } from './board/pack';
import type { CalendarMode } from './calendar/group';
import { CalendarGroup } from './calendar/group';
import { CaptureCard } from './cards/capture-card';
import { GrowthCard } from './cards/growth-card';
import { JournalCard } from './cards/journal-card';
import { LatestCard } from './cards/latest-card';
import { LibraryCard } from './cards/library-card';
import { PlanCard } from './cards/plan-card';
import { ReviewCard } from './cards/review-card';
import { TermCard } from './cards/term-card';
import { ToolboxCard } from './cards/toolbox-card';
import styles from './page.module.css';

const WEEK = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];

export function TodayPage() {
  const [mode, setMode] = useState<CalendarMode>('month');
  const [cursor, setCursor] = useState(() => startOfLocalMonth());
  const [selected, setSelected] = useState(() => startOfLocalDay());
  const selectedKey = toDateInputValue(selected);
  const dayQuery = useQuery(calendarRangeQuery(selectedKey, selectedKey));
  const day = dayQuery.data?.days[0];
  const cursorDate = new Date(cursor);
  const title =
    mode === 'year' ? `${cursorDate.getFullYear()}年` : `${cursorDate.getFullYear()}年${cursorDate.getMonth() + 1}月`;
  const holiday = day?.holidayName ?? '';
  const term = day?.lunar?.solarTerm ?? '';
  const extra = holiday && holiday !== term ? holiday : term;

  function selectDay(time: number) {
    const dayStart = startOfLocalDay(time);
    setSelected(dayStart);
    setMode('month');
    setCursor(startOfLocalMonth(dayStart));
  }

  function openMonth(monthStart: number) {
    setMode('month');
    setCursor(monthStart);
    const current = new Date(selected);
    const next = new Date(monthStart);
    if (current.getFullYear() !== next.getFullYear() || current.getMonth() !== next.getMonth()) {
      setSelected(monthStart);
    }
  }

  function showMonth() {
    setMode('month');
    setCursor(startOfLocalMonth(selected));
  }

  function showYear() {
    setMode('year');
    setCursor(startOfLocalYear(selected));
  }

  function shift(delta: number) {
    setCursor((current) => (mode === 'year' ? addLocalYears(current, delta) : addLocalMonths(current, delta)));
  }

  return (
    <section className={styles.page}>
      <Scroll className={styles.scroll} edge>
        <div className={styles.body}>
          <header className={styles.head}>
            <div className={styles.nav}>
              <IconButton label={mode === 'year' ? '上一年' : '上一月'} onClick={() => shift(-1)}>
                <CaretLeftIcon weight="regular" />
              </IconButton>
              <h1 className={styles.title}>{title}</h1>
              <IconButton label={mode === 'year' ? '下一年' : '下一月'} onClick={() => shift(1)}>
                <CaretRightIcon weight="regular" />
              </IconButton>
              <div className={styles.mode}>
                <button type="button" aria-pressed={mode === 'month'} onClick={showMonth}>
                  月
                </button>
                <button type="button" aria-pressed={mode === 'year'} onClick={showYear}>
                  年
                </button>
              </div>
            </div>
            <p className={styles.summary}>
              <strong>{formatMonthDay(selected)}</strong>
              <span>{WEEK[new Date(selected).getDay()]}</span>
              {day?.lunar?.text ? <span className={styles.muted}>{day.lunar.text}</span> : null}
              {extra ? <span className={styles.muted}>{extra}</span> : null}
              {day ? (
                <span className={styles.pill} data-rest={day.isWorkday ? 'false' : 'true'}>
                  {day.isWorkday ? '上班' : '休息'}
                </span>
              ) : null}
            </p>
          </header>
          <Board
            calendarMin={mode === 'year' ? YEAR_MIN : CAL_MIN}
            group={(columns) => (
              <CalendarGroup
                mode={mode}
                cursor={cursor}
                selected={selected}
                columns={columns}
                onSelect={selectDay}
                onOpenMonth={openMonth}
              />
            )}
            tiles={[
              { id: 'plan', node: <PlanCard /> },
              { id: 'journal', node: <JournalCard /> },
              { id: 'growth', node: <GrowthCard /> },
              { id: 'library', node: <LibraryCard /> },
              { id: 'review', node: <ReviewCard /> },
              { id: 'toolbox', node: <ToolboxCard />, static: true },
              { id: 'term', node: <TermCard selected={selected} />, static: true },
              { id: 'latest', node: <LatestCard /> },
              { id: 'capture', node: <CaptureCard />, static: true }
            ]}
          />
        </div>
      </Scroll>
    </section>
  );
}
