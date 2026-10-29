import clsx from 'clsx';
import { useLayoutEffect, useRef } from 'react';
import type { AlmanacHour } from '@/bridge/calendar';
import { Scroll } from '@/components/scroll';
import styles from './calendar.module.css';
import { CHEN_HOUR, currentHourIndex } from './hours';

type HoursCardProps = {
  hours: AlmanacHour[];
  today: boolean;
  pending: boolean;
  resetKey: string;
};

/** 今天把当前时辰滚到顶部，其他日子对齐辰时。底部留白，使亥时也能到顶。 */
export function HoursCard({ hours, today, pending, resetKey }: HoursCardProps) {
  const viewportRef = useRef<HTMLDivElement>(null);
  const target = today ? currentHourIndex() : CHEN_HOUR;
  const ordered = [...hours].sort((left, right) => left.hourIndex - right.hourIndex);

  useLayoutEffect(() => {
    const scroller = viewportRef.current;
    if (!scroller || resetKey.length === 0 || hours.length === 0) {
      return;
    }
    const align = () => {
      const row = scroller.querySelector<HTMLElement>(`[data-index="${target}"]`);
      const spacer = scroller.querySelector<HTMLElement>('[data-spacer]');
      if (!row || !spacer) {
        return;
      }
      spacer.style.height = `${Math.max(0, scroller.clientHeight - row.offsetHeight)}px`;
      scroller.scrollTop = row.offsetTop;
    };
    align();
    const observer = new ResizeObserver(align);
    observer.observe(scroller);
    return () => observer.disconnect();
  }, [target, resetKey, hours]);

  return (
    <article className={clsx(styles.card, styles.hours)}>
      <header className={styles.hoursHead}>
        <h2>时辰</h2>
        <span className={styles.hoursExtra}>上下滚动看全部</span>
      </header>
      <Scroll className={styles.hourScroll} axis="vertical" chain viewportRef={viewportRef}>
        {pending ? <p className={styles.note}>正在读取…</p> : null}
        {ordered.map((hour) => {
          const now = today && hour.hourIndex === target;
          return (
            <div
              key={hour.hourIndex}
              className={styles.hour}
              data-index={hour.hourIndex}
              data-now={now ? 'true' : undefined}
            >
              <span className={styles.zhi}>{hour.zhi}</span>
              <div>
                <p className={styles.hourMeta}>
                  <span>{hour.timeRange}</span>
                  {now ? <span className={styles.nowTag}>当前</span> : null}
                </p>
                <p className={styles.hourAdvice}>
                  <span className={styles.yiText}>宜 {hour.yi}</span>
                  <span className={styles.jiText}>忌 {hour.ji}</span>
                </p>
              </div>
            </div>
          );
        })}
        <div data-spacer="" aria-hidden="true" />
      </Scroll>
    </article>
  );
}
