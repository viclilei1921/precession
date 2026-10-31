import { useQuery } from '@tanstack/react-query';
import { adviceWords } from '@/features/today/calendar/hours';
import { Sheet } from '@/layout/mobile/sheet';
import { calendarAlmanacQuery } from '@/query/calendar/query';
import { formatDayHeading, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import styles from './almanac-sheet.module.css';

type AlmanacSheetProps = {
  day: number;
  onClose: () => void;
};

/** 长按日期后的宜忌。 */
export function AlmanacSheet({ day, onClose }: AlmanacSheetProps) {
  const query = useQuery(calendarAlmanacQuery(toDateInputValue(day)));
  const almanac = query.data;
  const yi = adviceWords(almanac?.yi ?? '');
  const ji = adviceWords(almanac?.ji ?? '');

  return (
    <Sheet title={formatDayHeading(day)} subtitle={almanac?.lunar.text || undefined} onClose={onClose}>
      {query.error ? <p className={styles.error}>{errorMessage(query.error)}</p> : null}
      {!almanac && !query.error ? (
        <p className={styles.note}>{query.isPending ? '正在读取…' : '这一天没有黄历。'}</p>
      ) : null}
      {almanac ? (
        <div className={styles.advice}>
          <Advice title="宜" tone="yi" words={yi.length > 0 ? yi : ['暂无']} />
          <Advice title="忌" tone="ji" words={ji.length > 0 ? ji : ['暂无']} />
        </div>
      ) : null}
    </Sheet>
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
          <li key={item.key}>{item.word}</li>
        ))}
      </ul>
    </section>
  );
}
