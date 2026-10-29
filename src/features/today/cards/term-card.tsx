import { SunIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useCardLayout } from '@/features/today/board/board';
import { termView } from '@/features/today/calendar/term';
import { calendarAlmanacQuery, calendarRangeQuery } from '@/query/calendar/query';
import { addLocalDays, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import styles from './tile.module.css';
import { TileHead } from './tile-head';

type TermCardProps = {
  selected: number;
};

export function TermCard({ selected }: TermCardProps) {
  const from = toDateInputValue(addLocalDays(selected, -24));
  const to = toDateInputValue(addLocalDays(selected, 24));
  const daysQuery = useQuery(calendarRangeQuery(from, to));
  const almanacQuery = useQuery(calendarAlmanacQuery(toDateInputValue(selected)));
  const view = termView(daysQuery.data?.days ?? [], selected, almanacQuery.data?.solarTerm ?? '');
  const error = daysQuery.error ?? almanacQuery.error;

  useCardLayout('term', `${daysQuery.isPending}:${view.name}:${view.hint}:${view.percent}`);

  return (
    <article className={styles.card} data-static="true">
      <TileHead tone="journal" icon={SunIcon} title="节气" />
      {error ? <p className={styles.error}>{errorMessage(error)}</p> : null}
      <p className={styles.figure}>{view.name}</p>
      <p className={styles.meta}>{view.hint}</p>
      <div className={styles.bar} role="img" aria-label="节气进度">
        <span data-tone="journal" style={{ width: `${view.percent}%` }} />
      </div>
    </article>
  );
}
