import clsx from 'clsx';
import { useLayoutEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import type { AlmanacHour } from '@/bridge/calendar';
import { Scroll } from '@/components/scroll';
import { translateYiJiText, translateZhi } from './almanac-dict';
import styles from './calendar.module.css';
import { CHEN_HOUR, currentHourIndex } from './hours';

type HoursCardProps = {
  hours: AlmanacHour[];
  today: boolean;
  pending: boolean;
  resetKey: string;
};

export function HoursCard({ hours, today, pending, resetKey }: HoursCardProps) {
  const { t } = useTranslation('today');
  const viewportRef = useRef<HTMLDivElement>(null);
  const target = today ? currentHourIndex() : CHEN_HOUR;
  const ordered = [...hours].sort((left, right) => left.hourIndex - right.hourIndex);
  const advice = (text: string) => translateYiJiText(text) || t('hours.none');

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
        <h2>{t('hours.title')}</h2>
        <span className={styles.hoursExtra}>{t('hours.hint')}</span>
      </header>
      <Scroll className={styles.hourScroll} axis="vertical" chain viewportRef={viewportRef}>
        {pending ? <p className={styles.note}>{t('common.loading')}</p> : null}
        {ordered.map((hour) => {
          const now = today && hour.hourIndex === target;
          return (
            <div
              key={hour.hourIndex}
              className={styles.hour}
              data-index={hour.hourIndex}
              data-now={now ? 'true' : undefined}
            >
              <span className={styles.zhi}>{translateZhi(hour.zhi)}</span>
              <div>
                <p className={styles.hourMeta}>
                  <span>{hour.timeRange}</span>
                  {now ? <span className={styles.nowTag}>{t('hours.current')}</span> : null}
                </p>
                <p className={styles.hourAdvice}>
                  <span className={styles.yiText}>{t('hours.yi', { text: advice(hour.yi) })}</span>
                  <span className={styles.jiText}>{t('hours.ji', { text: advice(hour.ji) })}</span>
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
