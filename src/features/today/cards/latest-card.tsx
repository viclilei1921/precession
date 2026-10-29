import { BookmarkSimpleIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import type { TimelineItem, TimelineKind } from '@/bridge/timeline';
import { useCardLayout } from '@/features/today/board/board';
import { timelineListQuery } from '@/query/timeline/query';
import { PATH } from '@/router/path';
import { addLocalDays, formatFeedClock, startOfLocalDay } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import styles from './tile.module.css';
import { TileHead } from './tile-head';

const kindLabel: Record<TimelineKind, string> = {
  plan: '计划',
  diary: '日记',
  spark: '灵感',
  writing: '写作',
  milestone: '里程碑',
  moment: '瞬间',
  excerpt: '书摘',
  note: '笔记'
};

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
  const navigate = useNavigate();
  const today = startOfLocalDay();
  const listQuery = useQuery(timelineListQuery(today, addLocalDays(today, 1)));
  const item = listQuery.data?.[0];
  const text = item ? item.title.trim() || item.body.trim() || '未命名' : '';

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
        title="最近一条"
        extra={item ? formatFeedClock(item.occurredAt) : undefined}
      />
      {listQuery.error ? <p className={styles.error}>{errorMessage(listQuery.error)}</p> : null}
      {item ? (
        <>
          <p className={styles.meta}>{kindLabel[item.kind]}</p>
          <p className={styles.quote}>{text}</p>
        </>
      ) : (
        <p className={styles.note}>{listQuery.isPending ? '正在读取…' : '今天还没有记录'}</p>
      )}
    </button>
  );
}
