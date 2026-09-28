import { CaretLeftIcon, CaretRightIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { useState } from 'react';
import type { TimelineKind } from '@/bridge/timeline';
import { PATH } from '@/router/path';
import { bookListQuery } from '@/shared/data/library/query';
import { timelineListQuery } from '@/shared/data/timeline/query';
import { addLocalYears, formatMonthDay, startOfLocalYear } from '@/shared/lib/day';
import { errorMessage } from '@/shared/lib/error';
import styles from '@/shared/ui/record.module.css';

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

export function YearPage() {
  const [yearStart, setYearStart] = useState(() => startOfLocalYear());
  const yearEnd = addLocalYears(yearStart, 1);
  const year = new Date(yearStart).getFullYear();
  const timelineQuery = useQuery(timelineListQuery(yearStart, yearEnd));
  const booksQuery = useQuery(bookListQuery);
  const items = timelineQuery.data ?? [];
  const highlights = items.filter((item) => item.highlight);
  const finished = (booksQuery.data ?? []).filter(
    (book) => book.finishedAt != null && book.finishedAt >= yearStart && book.finishedAt < yearEnd
  );
  const counts = new Map<TimelineKind, number>();
  for (const item of items) {
    counts.set(item.kind, (counts.get(item.kind) ?? 0) + 1);
  }

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>{year} 年度之书</h1>
        <p className={styles.hint}>这一年的记录自动汇总。高光来自手记、成长和书摘上的标记。</p>
        <div className={styles.actions}>
          <button
            type="button"
            className={styles.ghost}
            aria-label="上一年"
            onClick={() => setYearStart(addLocalYears(yearStart, -1))}
          >
            <CaretLeftIcon className={styles.icon} weight="regular" />
          </button>
          <button
            type="button"
            className={styles.ghost}
            aria-label="下一年"
            onClick={() => setYearStart(addLocalYears(yearStart, 1))}
          >
            <CaretRightIcon className={styles.icon} weight="regular" />
          </button>
        </div>
        <Link to={PATH.review} className={styles.primary}>
          返回回顾
        </Link>
      </header>
      {timelineQuery.error ? <p className={styles.error}>{errorMessage(timelineQuery.error)}</p> : null}
      <div className={styles.card}>
        <h2 className={styles.cardTitle}>这一年</h2>
        {items.length === 0 ? <p className={styles.empty}>还没有可以汇总的记录。</p> : null}
        <div className={styles.chips}>
          {(Object.keys(kindLabel) as TimelineKind[])
            .filter((kind) => counts.get(kind))
            .map((kind) => (
              <span key={kind} className={styles.chip} data-on="true">
                {kindLabel[kind]} {counts.get(kind)}
              </span>
            ))}
        </div>
      </div>
      <div className={styles.card}>
        <h2 className={styles.cardTitle}>高光</h2>
        {highlights.length === 0 ? <p className={styles.empty}>还没有标成高光的记录。</p> : null}
        {highlights.map((item) => (
          <div key={`${item.kind}-${item.id}`} className={styles.row}>
            <span className={styles.rowTitle}>
              {kindLabel[item.kind]} · {item.title.trim() || item.body.trim() || '未命名'}
            </span>
            <span className={styles.meta}>{formatMonthDay(item.occurredAt)}</span>
          </div>
        ))}
      </div>
      <div className={styles.card}>
        <h2 className={styles.cardTitle}>年度书单</h2>
        {finished.length === 0 ? <p className={styles.empty}>今年还没有标记读完的书。</p> : null}
        {finished.map((book) => (
          <div key={book.id} className={styles.row}>
            <span className={styles.rowTitle}>
              {book.title}
              {book.author ? ` · ${book.author}` : ''}
            </span>
            <span className={styles.meta}>{book.finishedAt != null ? formatMonthDay(book.finishedAt) : ''}</span>
          </div>
        ))}
      </div>
    </section>
  );
}
