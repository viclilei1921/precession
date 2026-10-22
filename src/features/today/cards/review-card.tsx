import { ClockIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useCardLayout } from '@/features/today/board/board';
import { timelineListQuery } from '@/query/timeline/query';
import { PATH } from '@/router/path';
import { addLocalDays, startOfLocalDay, startOfLocalWeek } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import styles from './tile.module.css';
import { TileHead } from './tile-head';

const WEEKS = 12;

function streakCount(days: Set<number>, todayStart: number): number {
  let cursor = days.has(todayStart) ? todayStart : addLocalDays(todayStart, -1);
  let count = 0;
  while (days.has(cursor)) {
    count += 1;
    cursor = addLocalDays(cursor, -1);
  }
  return count;
}

export function ReviewCard() {
  const navigate = useNavigate();
  const today = startOfLocalDay();
  const heatStart = addLocalDays(startOfLocalWeek(today), -(WEEKS - 1) * 7);
  const listQuery = useQuery(timelineListQuery(addLocalDays(today, -400), addLocalDays(today, 1)));
  const days = new Set((listQuery.data ?? []).map((item) => startOfLocalDay(item.occurredAt)));
  const streak = streakCount(days, today);
  const cells = Array.from({ length: WEEKS * 7 }, (_, index) => addLocalDays(heatStart, index));

  useCardLayout('review', `${listQuery.isPending}:${streak}:${listQuery.data?.length ?? 0}`);

  return (
    <button type="button" className={styles.card} onClick={() => void navigate({ to: PATH.review })}>
      <TileHead tone="journal" icon={ClockIcon} title="回顾" />
      {listQuery.error ? <p className={styles.error}>{errorMessage(listQuery.error)}</p> : null}
      <p className={styles.figure}>
        {streak}
        <small> 天</small>
      </p>
      <p className={styles.meta}>连续记录</p>
      <div className={styles.heat} aria-hidden="true">
        {cells.map((time) => (
          <i
            key={time}
            data-on={!listQuery.isPending && time <= today && days.has(time) ? 'true' : undefined}
            data-future={time > today ? 'true' : undefined}
          />
        ))}
      </div>
    </button>
  );
}
