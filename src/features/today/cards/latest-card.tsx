import { BookmarkSimpleIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useTranslation } from 'react-i18next';
import type { TimelineItem } from '@/bridge/timeline';
import { useCardLayout } from '@/features/today/board/board';
import { timelineListQuery } from '@/query/timeline/query';
import { PATH } from '@/router/path';
import { addLocalDays, formatFeedClock, startOfLocalDay } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import styles from './tile.module.css';
import { TileHead } from './tile-head';

function openItem(navigate: ReturnType<typeof useNavigate>, item: TimelineItem) {
  if (item.kind === 'plan') {
    void navigate({ to: PATH.plan, search: { id: item.id } });
    return;
  }
  if (item.kind === 'diary' || item.kind === 'spark' || item.kind === 'writing') {
    void navigate({ to: PATH.journalEntry, params: { entryId: item.id } });
    return;
  }
  if (item.kind === 'milestone' || item.kind === 'moment') {
    void navigate({ to: PATH.growth });
    return;
  }
  if (item.kind === 'excerpt' || item.kind === 'note') {
    void navigate({ to: PATH.library });
  }
}

export function LatestCard() {
  const { t } = useTranslation('today');
  const navigate = useNavigate();
  const today = startOfLocalDay();
  const listQuery = useQuery(timelineListQuery(today, addLocalDays(today, 1)));
  const item = listQuery.data?.[0];
  const text = item ? item.title.trim() || item.body.trim() || t('latestCard.untitled') : '';

  useCardLayout('latest', `${listQuery.isPending}:${item?.id ?? ''}:${text}`);

  return (
    <button
      type="button"
      className={styles.card}
      onClick={() => {
        if (item) {
          openItem(navigate, item);
        }
      }}
    >
      <TileHead
        tone="journal"
        icon={BookmarkSimpleIcon}
        title={t('latestCard.title')}
        extra={item ? formatFeedClock(item.occurredAt) : undefined}
      />
      {listQuery.error ? <p className={styles.error}>{errorMessage(listQuery.error)}</p> : null}
      {item ? (
        <>
          <p className={styles.meta}>{t(`latestCard.types.${item.kind}`)}</p>
          <p className={styles.quote}>{text}</p>
        </>
      ) : (
        <p className={styles.note}>{listQuery.isPending ? t('common.loading') : t('latestCard.noRecords')}</p>
      )}
    </button>
  );
}
