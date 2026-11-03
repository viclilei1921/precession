import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import type { AlmanacHour } from '@/bridge/calendar';
import { translateAlmanacWord, translateYiJiText, translateZhi } from '@/features/today/calendar/almanac-dict';
import { adviceWords, currentHourIndex } from '@/features/today/calendar/hours';
import { useBackLayer } from '@/layout/mobile/back';
import { Sheet } from '@/layout/mobile/sheet';
import { calendarAlmanacQuery } from '@/query/calendar/query';
import { formatDayHeading, isSameLocalDay, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import styles from './almanac-sheet.module.css';

type AlmanacSheetProps = {
  day: number;
  onClose: () => void;
};

/** 长按日期后的宜忌。 */
export function AlmanacSheet({ day, onClose }: AlmanacSheetProps) {
  const { t } = useTranslation('today');
  const query = useQuery(calendarAlmanacQuery(toDateInputValue(day)));
  const almanac = query.data;
  const yi = adviceWords(almanac?.yi ?? '');
  const ji = adviceWords(almanac?.ji ?? '');
  const today = isSameLocalDay(day, Date.now());
  const nowIndex = today ? currentHourIndex() : -1;
  const hours = [...(almanac?.hourYiJi ?? [])].sort((left, right) => left.hourIndex - right.hourIndex);
  useBackLayer(true, onClose);

  return (
    <Sheet title={formatDayHeading(day)} subtitle={almanac?.lunar.text || undefined} onClose={onClose}>
      {query.error ? <p className={styles.error}>{errorMessage(query.error)}</p> : null}
      {!almanac && !query.error ? (
        <p className={styles.note}>{query.isPending ? t('common.loading') : t('almanac.noData')}</p>
      ) : null}
      {almanac ? (
        <>
          <div className={styles.advice}>
            <Advice title={t('almanac.yi')} tone="yi" words={yi.length > 0 ? yi : [t('hours.none')]} />
            <Advice title={t('almanac.ji')} tone="ji" words={ji.length > 0 ? ji : [t('hours.none')]} />
          </div>
          <HourList hours={hours} nowIndex={nowIndex} t={t} />
        </>
      ) : null}
    </Sheet>
  );
}

type Translate = (key: string, options?: Record<string, unknown>) => string;

function HourList({ hours, nowIndex, t }: { hours: AlmanacHour[]; nowIndex: number; t: Translate }) {
  if (hours.length === 0) {
    return null;
  }
  return (
    <section className={styles.hours}>
      <h3>{t('hours.title')}</h3>
      <ul>
        {hours.map((hour) => {
          const now = hour.hourIndex === nowIndex;
          return (
            <li key={hour.hourIndex} data-now={now ? 'true' : undefined}>
              <span className={styles.zhi}>{translateZhi(hour.zhi)}</span>
              <span>
                <span className={styles.hourMeta}>
                  <span>{hour.timeRange}</span>
                  {now ? <span className={styles.nowTag}>{t('hours.current')}</span> : null}
                </span>
                <span className={styles.hourAdvice}>
                  <span className={styles.yiText}>
                    {t('hours.yi', { text: translateYiJiText(hour.yi) || t('hours.none') })}
                  </span>
                  <span className={styles.jiText}>
                    {t('hours.ji', { text: translateYiJiText(hour.ji) || t('hours.none') })}
                  </span>
                </span>
              </span>
            </li>
          );
        })}
      </ul>
    </section>
  );
}

function Advice({ title, tone, words }: { title: string; tone: 'yi' | 'ji'; words: string[] }) {
  const seen = new Map<string, number>();
  const items = words.map((word) => {
    const count = (seen.get(word) ?? 0) + 1;
    seen.set(word, count);
    return { key: count === 1 ? word : `${word}-${count}`, word };
  });

  return (
    <section className={styles[tone]}>
      <h3>{title}</h3>
      <ul>
        {items.map((item) => (
          <li key={item.key}>{translateAlmanacWord(item.word)}</li>
        ))}
      </ul>
    </section>
  );
}
