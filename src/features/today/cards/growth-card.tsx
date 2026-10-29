import { FlagIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useCardLayout } from '@/features/today/board/board';
import { growthListQuery } from '@/query/growth/query';
import { PATH } from '@/router/path';
import { formatMonthDay } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import styles from './tile.module.css';
import { TileHead } from './tile-head';

export function GrowthCard() {
  const navigate = useNavigate();
  const listQuery = useQuery(growthListQuery(undefined, 'milestone'));
  const latest = listQuery.data?.[0];
  const title = latest?.title.trim() || latest?.body.trim() || '';

  useCardLayout('growth', `${listQuery.isPending}:${latest?.id ?? ''}:${title}`);

  return (
    <button type="button" className={styles.card} onClick={() => void navigate({ to: PATH.growth })}>
      <TileHead tone="growth" icon={FlagIcon} title="成长" />
      {listQuery.error ? <p className={styles.error}>{errorMessage(listQuery.error)}</p> : null}
      {latest ? (
        <>
          <p className={styles.lead}>{title || '未命名'}</p>
          <p className={styles.meta}>{formatMonthDay(latest.occurredAt)}</p>
        </>
      ) : (
        <p className={styles.note}>{listQuery.isPending ? '正在读取…' : '还没有里程碑'}</p>
      )}
    </button>
  );
}
