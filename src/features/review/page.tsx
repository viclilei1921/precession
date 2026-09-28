import { CaretLeft, CaretRight } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { Link, useNavigate } from '@tanstack/react-router';
import { useState } from 'react';
import type { TimelineKind } from '@/bridge/timeline';
import { PATH } from '@/router/path';
import { timelineListQuery } from '@/shared/data/timeline/query';
import { addLocalMonths, formatFeedClock, formatMonth, formatMonthDay, startOfLocalMonth } from '@/shared/lib/day';
import { errorMessage } from '@/shared/lib/error';
import styles from '@/shared/ui/record.module.css';

const filters: { kind: TimelineKind | 'all'; label: string }[] = [
  { kind: 'all', label: '全部' },
  { kind: 'plan', label: '计划' },
  { kind: 'diary', label: '日记' },
  { kind: 'spark', label: '灵感' },
  { kind: 'writing', label: '写作' },
  { kind: 'milestone', label: '里程碑' },
  { kind: 'moment', label: '瞬间' },
  { kind: 'excerpt', label: '书摘' },
  { kind: 'note', label: '笔记' }
];

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

export function ReviewPage() {
  const navigate = useNavigate();
  const [month, setMonth] = useState(() => startOfLocalMonth());
  const [kind, setKind] = useState<TimelineKind | 'all'>('all');
  const from = month;
  const to = addLocalMonths(month, 1);
  const listQuery = useQuery(timelineListQuery(from, to, kind === 'all' ? undefined : [kind]));
  const items = listQuery.data ?? [];

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>回顾</h1>
        <p className={styles.hint}>把四块记录按时间再读一遍。</p>
        <div className={styles.actions}>
          <button
            type="button"
            className={styles.ghost}
            aria-label="上个月"
            onClick={() => setMonth(addLocalMonths(month, -1))}
          >
            <CaretLeft className={styles.icon} weight="regular" />
          </button>
          <span>{formatMonth(month)}</span>
          <button
            type="button"
            className={styles.ghost}
            aria-label="下个月"
            onClick={() => setMonth(addLocalMonths(month, 1))}
          >
            <CaretRight className={styles.icon} weight="regular" />
          </button>
        </div>
        <Link to={PATH.yearBook} className={styles.primary}>
          年度之书
        </Link>
      </header>
      <div className={styles.chips}>
        {filters.map((item) => (
          <button
            key={item.kind}
            type="button"
            className={styles.chip}
            data-on={kind === item.kind ? 'true' : undefined}
            onClick={() => setKind(item.kind)}
          >
            {item.label}
          </button>
        ))}
      </div>
      {listQuery.error ? <p className={styles.error}>{errorMessage(listQuery.error)}</p> : null}
      <div className={styles.card}>
        {listQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}
        {!listQuery.isPending && items.length === 0 ? <p className={styles.empty}>这个月还没有记录。</p> : null}
        {items.map((item) => (
          <button
            key={`${item.kind}-${item.id}`}
            type="button"
            className={styles.row}
            onClick={() => openItem(navigate, item.kind, item.id)}
          >
            <span className={styles.rowTitle}>
              {kindLabel[item.kind]} · {item.title.trim() || item.body.trim() || '未命名'}
            </span>
            <span className={styles.meta}>
              {item.highlight ? '高光 · ' : ''}
              {formatMonthDay(item.occurredAt)} {formatFeedClock(item.occurredAt)}
            </span>
          </button>
        ))}
      </div>
    </section>
  );
}

function openItem(navigate: ReturnType<typeof useNavigate>, kind: TimelineKind, id: string) {
  if (kind === 'plan') {
    void navigate({ to: PATH.plan, search: { id } });
    return;
  }
  if (kind === 'diary' || kind === 'spark' || kind === 'writing') {
    void navigate({ to: PATH.journalEntry, params: { entryId: id } });
    return;
  }
  if (kind === 'milestone' || kind === 'moment') {
    void navigate({ to: PATH.growth });
    return;
  }
  void navigate({ to: PATH.library });
}
