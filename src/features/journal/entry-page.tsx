import { CaretLeftIcon, PencilSimpleIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useParams } from '@tanstack/react-router';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { IconButton } from '@/components/button/icon';
import { MarkdownEditor } from '@/components/markdown';
import { popRoute } from '@/layout/mobile/back';
import { journalEntryQuery } from '@/query/journal/query';
import { PATH } from '@/router/path';
import { errorMessage } from '@/utils/error';
import { JournalEditor } from './components/editor';
import styles from './entry.module.css';
import { useJournalNames } from './names';
import { journalDate, journalWords, lookupNames } from './text';

/** 一条手记。已写的先阅读，编辑时再展开不常改的项。 */
export function JournalEntryPage() {
  const { t } = useTranslation(['journal', 'common']);
  const { entryId } = useParams({ from: '/journal/$entryId' });
  const navigate = useNavigate();
  const entryQuery = useQuery(journalEntryQuery(entryId));
  const names = useJournalNames();
  const [editing, setEditing] = useState(false);

  if (entryQuery.isPending) {
    return (
      <section className={styles.page}>
        <p className={styles.note}>{t('common:state.loading')}</p>
      </section>
    );
  }
  if (entryQuery.error || !entryQuery.data) {
    return (
      <section className={styles.page}>
        <p className={styles.error}>{errorMessage(entryQuery.error ?? t('errors.notFound'))}</p>
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
        onDeleted={() =>
          popRoute(() => void navigate({ to: PATH.journal, search: { kind: entry.kind }, replace: true }))
        }
      />
    );
  }

  const words = journalWords(entry.body, entry.locked);
  const byline = [
    journalDate(entry.occurredAt).full,
    entry.highlight ? t('page.highlight') : '',
    ...lookupNames(entry.placeIds, names.places),
    ...lookupNames(entry.memberIds, names.members),
    ...lookupNames(entry.tagIds, names.tags),
    words == null ? '' : t('page.words', { count: words })
  ]
    .filter((item) => item.length > 0)
    .join(' · ');

  return (
    <section className={styles.readPage}>
      <div className={styles.chrome}>
        <IconButton
          label={t('common:button.back')}
          variant="outlined"
          className={styles.float}
          onClick={() =>
            popRoute(() => void navigate({ to: PATH.journal, search: { kind: entry.kind }, replace: true }))
          }
        >
          <CaretLeftIcon weight="regular" />
        </IconButton>
        <IconButton
          label={t('entry.edit')}
          variant="outlined"
          className={styles.float}
          onClick={() => setEditing(true)}
        >
          <PencilSimpleIcon weight="regular" />
        </IconButton>
      </div>
      <div className={styles.read}>
        <h1 className={styles.readTitle}>{entry.title}</h1>
        <p className={styles.byline}>{byline}</p>
        {entry.locked ? (
          <p className={styles.notice}>{t('editor.lockedNotice')}</p>
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
