import { CaretLeftIcon, PencilSimpleIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useParams } from '@tanstack/react-router';
import { useState } from 'react';
import { IconButton } from '@/components/button/icon';
import { MarkdownEditor } from '@/components/markdown';
import { journalEntryQuery } from '@/query/journal/query';
import { PATH } from '@/router/path';
import { errorMessage } from '@/utils/error';
import { JournalEditor } from './components/editor';
import styles from './entry.module.css';
import { useJournalNames } from './names';
import { journalDate, journalWords, lookupNames } from './text';

/** 一条手记。已写的先阅读，编辑时再展开不常改的项。 */
export function JournalEntryPage() {
  const { entryId } = useParams({ from: '/journal/$entryId' });
  const navigate = useNavigate();
  const entryQuery = useQuery(journalEntryQuery(entryId));
  const names = useJournalNames();
  const [editing, setEditing] = useState(false);

  if (entryQuery.isPending) {
    return (
      <section className={styles.page}>
        <p className={styles.note}>正在读取…</p>
      </section>
    );
  }
  if (entryQuery.error || !entryQuery.data) {
    return (
      <section className={styles.page}>
        <p className={styles.error}>{errorMessage(entryQuery.error ?? '手记不存在')}</p>
      </section>
    );
  }

  const entry = entryQuery.data;
  if (editing) {
    return (
      <JournalEditor
        key={entry.updatedAt}
        entry={entry}
        kind={entry.kind}
        onClose={() => setEditing(false)}
        onSaved={() => setEditing(false)}
        onDeleted={() => void navigate({ to: PATH.journal, search: { kind: entry.kind } })}
      />
    );
  }

  const words = journalWords(entry.body, entry.locked);
  const byline = [
    journalDate(entry.occurredAt).full,
    entry.highlight ? '高光' : '',
    ...lookupNames(entry.placeIds, names.places),
    ...lookupNames(entry.memberIds, names.members),
    ...lookupNames(entry.tagIds, names.tags),
    words == null ? '' : `${words} 字`
  ]
    .filter((item) => item.length > 0)
    .join(' · ');

  return (
    <section className={styles.readPage}>
      <div className={styles.chrome}>
        <IconButton
          label="返回"
          variant="outlined"
          className={styles.float}
          onClick={() => void navigate({ to: PATH.journal, search: { kind: entry.kind } })}
        >
          <CaretLeftIcon weight="regular" />
        </IconButton>
        <IconButton label="编辑" variant="outlined" className={styles.float} onClick={() => setEditing(true)}>
          <PencilSimpleIcon weight="regular" />
        </IconButton>
      </div>
      <div className={styles.read}>
        <h1 className={styles.readTitle}>{entry.title}</h1>
        <p className={styles.byline}>{byline}</p>
        {entry.locked ? (
          <p className={styles.notice}>正文已封存。解开之后才能改。</p>
        ) : (
          <div className={styles.readBody}>
            <MarkdownEditor
              key={entry.updatedAt}
              className={styles.fill}
              height="100%"
              tools={false}
              edge
              disabled
              value={entry.body}
              image={{ owner: 'journal_entry', ownerId: entry.id }}
              onChange={() => undefined}
            />
          </div>
        )}
      </div>
    </section>
  );
}
