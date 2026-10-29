import { NotebookIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useCardLayout } from '@/features/today/board/board';
import { timelineListQuery } from '@/query/timeline/query';
import { PATH } from '@/router/path';
import { addLocalDays, startOfLocalDay } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import styles from './tile.module.css';
import { TileHead } from './tile-head';

const kinds: { kind: 'diary' | 'spark' | 'writing'; label: string }[] = [
  { kind: 'diary', label: '日记' },
  { kind: 'spark', label: '灵感' },
  { kind: 'writing', label: '写作' }
];

export function JournalCard() {
  const navigate = useNavigate();
  const today = startOfLocalDay();
  const listQuery = useQuery(timelineListQuery(today, addLocalDays(today, 1), ['diary', 'spark', 'writing']));
  const items = listQuery.data ?? [];
  const counts = kinds.map((item) => items.filter((entry) => entry.kind === item.kind).length);

  useCardLayout('journal', `${listQuery.isPending}:${counts.join(',')}`);

  return (
    <article className={styles.card}>
      <button type="button" className={styles.jump} onClick={() => void navigate({ to: PATH.journal })}>
        <TileHead tone="journal" icon={NotebookIcon} title="手记" />
      </button>
      {listQuery.error ? <p className={styles.error}>{errorMessage(listQuery.error)}</p> : null}
      <ul className={styles.counts}>
        {kinds.map((item, index) => (
          <li key={item.kind}>
            <button type="button" onClick={() => void navigate({ to: PATH.journal, search: { kind: item.kind } })}>
              <strong>{counts[index]}</strong>
              <span>{item.label}</span>
            </button>
          </li>
        ))}
      </ul>
    </article>
  );
}
