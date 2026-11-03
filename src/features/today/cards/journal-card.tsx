import { NotebookIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useTranslation } from 'react-i18next';
import { useCardLayout } from '@/features/today/board/board';
import { timelineListQuery } from '@/query/timeline/query';
import { PATH } from '@/router/path';
import { addLocalDays, startOfLocalDay } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import styles from './tile.module.css';
import { TileHead } from './tile-head';

export function JournalCard() {
  const { t } = useTranslation('today');
  const navigate = useNavigate();
  const today = startOfLocalDay();
  const listQuery = useQuery(timelineListQuery(today, addLocalDays(today, 1), ['diary', 'spark', 'writing']));
  const items = listQuery.data ?? [];

  const kinds: { kind: 'diary' | 'spark' | 'writing'; label: string }[] = [
    { kind: 'diary', label: t('journalCard.diary') },
    { kind: 'spark', label: t('journalCard.spark') },
    { kind: 'writing', label: t('journalCard.writing') }
  ];
  const counts = kinds.map((item) => items.filter((entry) => entry.kind === item.kind).length);

  useCardLayout('journal', `${listQuery.isPending}:${counts.join(',')}`);

  return (
    <article className={styles.card}>
      <button type="button" className={styles.jump} onClick={() => void navigate({ to: PATH.journal })}>
        <TileHead tone="journal" icon={NotebookIcon} title={t('card.journal')} />
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
