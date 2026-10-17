import type { Icon } from '@phosphor-icons/react';
import {
  BooksIcon,
  CameraIcon,
  CaretLeftIcon,
  CaretRightIcon,
  FlagIcon,
  LightningIcon,
  ListChecksIcon,
  NotebookIcon,
  NoteIcon,
  PencilLineIcon,
  QuotesIcon,
  SquaresFourIcon
} from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useState } from 'react';
import type { TimelineKind } from '@/bridge/timeline';
import { ChipButton } from '@/components/button/chip';
import { IconButton } from '@/components/button/icon';
import { LinkButton } from '@/components/button/link';
import { RowButton } from '@/components/button/row';
import styles from '@/components/record.module.css';
import { Scroll } from '@/components/scroll';
import { usePageSearch } from '@/hooks/search';
import { timelineListQuery } from '@/query/timeline/query';
import { PATH } from '@/router/path';
import { addLocalMonths, formatFeedClock, formatMonth, formatMonthDay, startOfLocalMonth } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import { matchesQuery } from '@/utils/search';

const filters: { kind: TimelineKind | 'all'; label: string; Icon: Icon }[] = [
  { kind: 'all', label: '全部', Icon: SquaresFourIcon },
  { kind: 'plan', label: '计划', Icon: ListChecksIcon },
  { kind: 'diary', label: '日记', Icon: NotebookIcon },
  { kind: 'spark', label: '灵感', Icon: LightningIcon },
  { kind: 'writing', label: '写作', Icon: PencilLineIcon },
  { kind: 'milestone', label: '里程碑', Icon: FlagIcon },
  { kind: 'moment', label: '瞬间', Icon: CameraIcon },
  { kind: 'excerpt', label: '书摘', Icon: QuotesIcon },
  { kind: 'note', label: '笔记', Icon: NoteIcon }
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
  const query = usePageSearch('搜索回顾');
  const items = (listQuery.data ?? []).filter((item) =>
    matchesQuery(query, item.title, item.body, kindLabel[item.kind])
  );

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>回顾</h1>
        <p className={styles.hint}>把四块记录按时间再读一遍。</p>
        <div className={styles.actions}>
          <IconButton label="上个月" onClick={() => setMonth(addLocalMonths(month, -1))}>
            <CaretLeftIcon weight="regular" />
          </IconButton>
          <span>{formatMonth(month)}</span>
          <IconButton label="下个月" onClick={() => setMonth(addLocalMonths(month, 1))}>
            <CaretRightIcon weight="regular" />
          </IconButton>
        </div>
        <LinkButton to={PATH.yearBook} className={styles.trailing}>
          <BooksIcon weight="regular" />
          年度之书
        </LinkButton>
      </header>
      <Scroll className={styles.scroll}>
        <div className={styles.body}>
          <div className={styles.chips}>
            {filters.map((item) => (
              <ChipButton key={item.kind} selected={kind === item.kind} onClick={() => setKind(item.kind)}>
                <item.Icon weight="regular" />
                {item.label}
              </ChipButton>
            ))}
          </div>
          {listQuery.error ? <p className={styles.error}>{errorMessage(listQuery.error)}</p> : null}
          <div className={styles.card}>
            {listQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}
            {!listQuery.isPending && items.length === 0 ? (
              <p className={styles.empty}>{query.trim() ? '没有匹配的记录。' : '这个月还没有记录。'}</p>
            ) : null}
            {items.map((item) => (
              <RowButton key={`${item.kind}-${item.id}`} onClick={() => openItem(navigate, item.kind, item.id)}>
                <span className={styles.rowTitle}>
                  {kindLabel[item.kind]} · {item.title.trim() || item.body.trim() || '未命名'}
                </span>
                <span className={styles.meta}>
                  {item.highlight ? '高光 · ' : ''}
                  {formatMonthDay(item.occurredAt)} {formatFeedClock(item.occurredAt)}
                </span>
              </RowButton>
            ))}
          </div>
        </div>
      </Scroll>
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
