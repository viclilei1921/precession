import { CaretLeftIcon, PencilSimpleIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useParams } from '@tanstack/react-router';
import { useState } from 'react';
import { IconButton } from '@/components/button/icon';
import { MarkdownEditor } from '@/components/markdown';
import { useJournalNames } from '@/features/journal/names';
import { journalDate, journalWords, lookupNames } from '@/features/journal/text';
import { growthEntryQuery } from '@/query/growth/query';
import { PATH } from '@/router/path';
import { errorMessage } from '@/utils/error';
import { GrowthEditor } from './components/editor';
import styles from './entry.module.css';
import { cardAge, isYoung } from './text';

/** 一条成长记录。已写的先阅读，编辑时再打开抽屉。 */
export function GrowthEntryPage() {
  const { entryId } = useParams({ from: '/growth/$entryId' });
  const navigate = useNavigate();
  const entryQuery = useQuery(growthEntryQuery(entryId));
  const names = useJournalNames();
  const [editing, setEditing] = useState(false);

  function back(memberId: string) {
    void navigate({ to: PATH.growth, search: { member: memberId } });
  }

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
        <p className={styles.error}>{errorMessage(entryQuery.error ?? '成长记录不存在')}</p>
      </section>
    );
  }

  const entry = entryQuery.data;
  if (editing) {
    return (
      <GrowthEditor
        key={entry.updatedAt}
        entry={entry}
        ownerId={entry.memberId}
        kind={entry.kind}
        onClose={() => setEditing(false)}
        onSaved={() => setEditing(false)}
        onDeleted={() => back(entry.memberId)}
      />
    );
  }

  const person = names.members.find((item) => item.id === entry.memberId);
  const young = isYoung(person?.birthday ?? null);
  const words = journalWords(entry.body, entry.locked);
  const byline = [
    journalDate(entry.occurredAt).full,
    person?.name ?? '',
    cardAge(person?.birthday ?? null, entry.occurredAt, young),
    entry.kind === 'milestone' ? '里程碑' : '瞬间',
    entry.highlight ? '高光' : '',
    entry.locked ? '已封存' : '',
    ...lookupNames(entry.placeIds, names.places),
    ...lookupNames(
      entry.memberIds.filter((id) => id !== entry.memberId),
      names.members
    ),
    ...lookupNames(entry.tagIds, names.tags),
    words == null ? '' : `${words} 字`
  ]
    .filter((item) => item.length > 0)
    .join(' · ');

  return (
    <section className={styles.readPage}>
      <div className={styles.chrome}>
        <IconButton label="返回" variant="outlined" className={styles.float} onClick={() => back(entry.memberId)}>
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
          <p className={styles.note}>正文已封存。</p>
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
              image={{ owner: 'growth_entry', ownerId: entry.id }}
              onChange={() => undefined}
            />
          </div>
        )}
      </div>
    </section>
  );
}
