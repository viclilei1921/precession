import { LightningIcon, NotebookIcon, PencilLineIcon, PlusIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import type { JournalKind } from '@/bridge/journal';
import { Dialog } from '@/components/dialog';
import styles from '@/components/record.module.css';
import { usePageSearch } from '@/layout/page-search';
import { journalListQuery } from '@/query/journal/query';
import { PATH } from '@/router/path';
import { formatMonthDay } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import { matchesQuery } from '@/utils/search';
import { JournalEditor } from './editor';

const kinds: { kind: JournalKind; label: string; Icon: typeof NotebookIcon }[] = [
  { kind: 'diary', label: '日记', Icon: NotebookIcon },
  { kind: 'spark', label: '灵感', Icon: LightningIcon },
  { kind: 'writing', label: '写作', Icon: PencilLineIcon }
];

export function JournalPage() {
  const navigate = useNavigate();
  const search = useSearch({ from: '/journal' });
  const [kind, setKind] = useState<JournalKind>(search.kind ?? 'diary');
  const [creating, setCreating] = useState(false);
  const listQuery = useQuery(journalListQuery(kind));
  const query = usePageSearch('搜索手记');
  const entries = [...(listQuery.data ?? [])]
    .filter((entry) => matchesQuery(query, entry.title, entry.body))
    .sort((left, right) => right.occurredAt - left.occurredAt);

  useEffect(() => {
    if (search.kind) {
      setKind(search.kind);
    }
    if (search.create) {
      setCreating(true);
    }
    if (search.create || search.kind) {
      void navigate({ to: PATH.journal, search: {}, replace: true });
    }
  }, [navigate, search.create, search.kind]);

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>手记</h1>
        <p className={styles.hint}>日记、灵感和写作。灵感可以扩写成文，再归入某一天的日记。</p>
        <div className={styles.segments} role="tablist" aria-label="手记类型">
          {kinds.map((item) => (
            <button
              key={item.kind}
              type="button"
              className={styles.segment}
              data-on={kind === item.kind ? 'true' : undefined}
              onClick={() => setKind(item.kind)}
            >
              <item.Icon className={styles.icon} weight="regular" />
              {item.label}
            </button>
          ))}
        </div>
        <button type="button" className={styles.primary} onClick={() => setCreating(true)}>
          <PlusIcon className={styles.icon} weight="regular" />
          新建
        </button>
      </header>
      {listQuery.error ? <p className={styles.error}>{errorMessage(listQuery.error)}</p> : null}
      <div className={styles.card}>
        {listQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}
        {!listQuery.isPending && entries.length === 0 ? (
          <p className={styles.empty}>{query.trim() ? '没有匹配的手记。' : '这里还没有记录。'}</p>
        ) : null}
        {entries.map((entry) => (
          <button
            key={entry.id}
            type="button"
            className={styles.row}
            onClick={() => void navigate({ to: PATH.journalEntry, params: { entryId: entry.id } })}
          >
            <span className={styles.rowTitle}>{entry.title}</span>
            <span className={styles.meta}>
              {entry.highlight ? '高光 · ' : ''}
              {entry.locked ? '已锁定 · ' : ''}
              {formatMonthDay(entry.occurredAt)}
            </span>
          </button>
        ))}
      </div>
      {creating ? (
        <Dialog title="新建手记" onClose={() => setCreating(false)}>
          <JournalEditor
            entry={null}
            kind={kind}
            onClose={() => setCreating(false)}
            onSaved={(entry) => {
              setCreating(false);
              void navigate({ to: PATH.journalEntry, params: { entryId: entry.id } });
            }}
          />
        </Dialog>
      ) : null}
    </section>
  );
}
