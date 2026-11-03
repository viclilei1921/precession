import { CaretLeftIcon, CaretRightIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
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
import { GrowthCard } from './cards/growth-card';
import { JournalCard } from './cards/journal-card';
import { LatestCard } from './cards/latest-card';
import { LibraryCard } from './cards/library-card';
import { PlanCard } from './cards/plan-card';
import { TermCard } from './cards/term-card';
import { ToolboxCard } from './cards/toolbox-card';
import styles from './page.module.css';

export function TodayPage() {
  const { t } = useTranslation('today');
  const [mode, setMode] = useState<CalendarMode>('month');
  const [cursor, setCursor] = useState(() => startOfLocalMonth());
  const [selected, setSelected] = useState(() => startOfLocalDay());
  const selectedKey = toDateInputValue(selected);
  const dayQuery = useQuery(calendarRangeQuery(selectedKey, selectedKey));
  const day = dayQuery.data?.days[0];
  const cursorDate = new Date(cursor);
  const title =
    mode === 'year'
      ? t('page.yearTitle', { year: cursorDate.getFullYear() })
      : t('page.monthTitle', { year: cursorDate.getFullYear(), month: cursorDate.getMonth() + 1 });
  const holiday = day?.holidayName ?? '';
  const term = day?.lunar?.solarTerm ?? '';
  const extra = holiday && holiday !== term ? holiday : term;

  const WEEK_KEYS = ['sun', 'mon', 'tue', 'wed', 'thu', 'fri', 'sat'] as const;

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
              <IconButton label={mode === 'year' ? t('page.prevYear') : t('page.prevMonth')} onClick={() => shift(-1)}>
                <CaretLeftIcon weight="regular" />
              </IconButton>
              <h1 className={styles.title}>{title}</h1>
              <IconButton label={mode === 'year' ? t('page.nextYear') : t('page.nextMonth')} onClick={() => shift(1)}>
                <CaretRightIcon weight="regular" />
              </IconButton>
              <div className={styles.mode}>
                <button type="button" aria-pressed={mode === 'month'} onClick={showMonth}>
                  {t('page.monthView')}
                </button>
                <button type="button" aria-pressed={mode === 'year'} onClick={showYear}>
                  {t('page.yearView')}
                </button>
              </div>
            </div>
            <p className={styles.summary}>
              <strong>{formatMonthDay(selected)}</strong>
              <span>{t(`weekday.${WEEK_KEYS[new Date(selected).getDay()]}`)}</span>
              {day?.lunar?.text ? <span className={styles.muted}>{day.lunar.text}</span> : null}
              {extra ? <span className={styles.muted}>{extra}</span> : null}
              {day ? (
                <span className={styles.pill} data-rest={day.isWorkday ? 'false' : 'true'}>
                  {day.isWorkday ? t('page.workday') : t('page.rest')}
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
              { id: 'journal', node: <JournalCard />, static: true },
              { id: 'growth', node: <GrowthCard /> },
              { id: 'library', node: <LibraryCard /> },
              { id: 'toolbox', node: <ToolboxCard />, static: true },
              { id: 'term', node: <TermCard selected={selected} />, static: true },
              { id: 'latest', node: <LatestCard /> }
            ]}
          />
        </div>
      </Scroll>
    </section>
  );
}
