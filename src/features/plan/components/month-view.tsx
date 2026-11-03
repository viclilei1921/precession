import type { CSSProperties } from 'react';
import { useLayoutEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { Plan } from '@/bridge/plan';
import { formatMonthDay, startOfLocalDay } from '@/utils/day';
import styles from '../calendar.module.css';
import { planShowTime } from '../format';
import { isWeekend, layoutBars, monthWeeks } from '../layout';
import { PlanChip } from './plan-chip';

type MonthViewProps = {
  month: number;
  today: number;
  plans: Plan[];
  hueOf: (plan: Plan) => number;
  onCreate: (day: number) => void;
  onOpen: (plan: Plan) => void;
  onToggle: (plan: Plan) => void;
};

const LANE = 26;
const TOP = 28;
const MORE = 20;
const WEEKDAY_KEYS = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'] as const;

/** 月历。六周均分高度，排不下的收成个数 */
export function MonthView({ month, today, plans, hueOf, onCreate, onOpen, onToggle }: MonthViewProps) {
  const { t } = useTranslation('plan');
  const monthDate = new Date(month);
  const weeks = monthWeeks(month);
  const weeksRef = useRef<HTMLDivElement>(null);
  const [room, setRoom] = useState(0);

  useLayoutEffect(() => {
    const root = weeksRef.current;
    if (!root) {
      return;
    }
    const measure = () => {
      const week = root.firstElementChild;
      const height = week instanceof HTMLElement ? week.clientHeight : 0;
      setRoom((current) => (current === height ? current : height));
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(root);
    return () => observer.disconnect();
  }, []);

  return (
    <div className={styles.month}>
      <div className={styles.weekdays}>
        {WEEKDAY_KEYS.map((key, index) => (
          <span key={key} data-weekend={index >= 5 ? 'true' : undefined}>
            {t(`weekday.${key}`)}
          </span>
        ))}
      </div>
      <div ref={weeksRef} className={styles.weeks}>
        {weeks.map((days) => {
          const segments = layoutBars(days, plans, true);
          const lanes = segments.reduce((max, segment) => Math.max(max, segment.lane + 1), 0);
          const limit = laneLimit(room, lanes);
          const visible = segments.filter((segment) => segment.lane < limit);
          return (
            <div key={days[0]} className={styles.week}>
              {days.map((day) => {
                const date = new Date(day);
                const outside = date.getMonth() !== monthDate.getMonth();
                const label = date.getDate() === 1 ? formatMonthDay(day) : String(date.getDate());
                return (
                  <div
                    key={day}
                    className={styles.cell}
                    data-out={outside ? 'true' : undefined}
                    data-weekend={isWeekend(day) ? 'true' : undefined}
                  >
                    <button
                      type="button"
                      className={styles.cellHit}
                      aria-label={t('aria.addPlan', { date: formatMonthDay(day) })}
                      aria-current={startOfLocalDay(day) === today ? 'date' : undefined}
                      onClick={() => onCreate(day)}
                    />
                    <span className={styles.num} data-today={startOfLocalDay(day) === today ? 'true' : undefined}>
                      {label}
                    </span>
                  </div>
                );
              })}
              {visible.map((segment) => (
                <PlanChip
                  key={`${segment.plan.id}-${segment.col}`}
                  plan={segment.plan}
                  hue={hueOf(segment.plan)}
                  time={segment.showTime ? planShowTime(segment.plan, t) : undefined}
                  className={styles.bar}
                  before={segment.before}
                  after={segment.after}
                  style={barStyle(segment.col, segment.span, segment.lane, segment.before, segment.after)}
                  onOpen={onOpen}
                  onToggle={onToggle}
                />
              ))}
              {days.map((day, col) => {
                const hidden = segments.filter(
                  (segment) => segment.lane >= limit && col >= segment.col && col < segment.col + segment.span
                ).length;
                if (!hidden) {
                  return null;
                }
                return (
                  <span
                    key={day}
                    className={styles.more}
                    style={{ top: TOP + limit * LANE, left: `calc(${(col / 7) * 100}% + 4px)` }}
                  >
                    +{hidden}
                  </span>
                );
              })}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function laneLimit(room: number, lanes: number): number {
  const fit = Math.max(0, Math.floor((room - TOP) / LANE));
  if (lanes <= fit) {
    return lanes;
  }
  return Math.max(0, Math.floor((room - TOP - MORE) / LANE));
}

function barStyle(col: number, span: number, lane: number, before: boolean, after: boolean): CSSProperties {
  const leftInset = before ? 0 : 4;
  const rightInset = after ? 0 : 4;
  return {
    top: TOP + lane * LANE,
    left: `calc(${(col / 7) * 100}% + ${leftInset}px)`,
    width: `calc(${(span / 7) * 100}% - ${leftInset + rightInset}px)`
  };
}
