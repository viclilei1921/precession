import { CaretLeftIcon, PencilSimpleIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useParams } from '@tanstack/react-router';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { IconButton } from '@/components/button/icon';
import { MarkdownEditor } from '@/components/markdown';
import { useJournalNames } from '@/features/journal/names';
import { journalWords, lookupNames } from '@/features/journal/text';
import { popRoute } from '@/layout/mobile/back';
import { growthEntryQuery } from '@/query/growth/query';
import { PATH } from '@/router/path';
import { formatDate } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import { GrowthEditor } from './components/editor';
import styles from './entry.module.css';
import { cardAge, isYoung } from './text';

/** 一条成长记录。已写的先阅读，编辑时再打开抽屉。 */
export function GrowthEntryPage() {
  const { t } = useTranslation('growth');
  const { entryId } = useParams({ from: '/growth/$entryId' });
  const navigate = useNavigate();
  const entryQuery = useQuery(growthEntryQuery(entryId));
  const names = useJournalNames();
  const [editing, setEditing] = useState(false);

  function back(memberId: string) {
    popRoute(() => {
      void navigate({ to: PATH.growth, search: { member: memberId }, replace: true });
    });
  }

  if (entryQuery.isPending) {
    return (
      <section className={styles.page}>
        <p className={styles.note}>{t('page.loading')}</p>
      </section>
    );
  }
  if (entryQuery.error || !entryQuery.data) {
    return (
      <section className={styles.page}>
        <p className={styles.error}>{errorMessage(entryQuery.error ?? t('page.notFound'))}</p>
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
    formatDate(entry.occurredAt),
    person?.name ?? '',
    cardAge(person?.birthday ?? null, entry.occurredAt, young),
    entry.kind === 'milestone' ? t('page.milestone') : t('page.moment'),
    entry.highlight ? t('page.highlight') : '',
    entry.locked ? t('page.locked') : '',
    ...lookupNames(entry.placeIds, names.places),
    ...lookupNames(
      entry.memberIds.filter((id) => id !== entry.memberId),
      names.members
    ),
    ...lookupNames(entry.tagIds, names.tags),
    words == null ? '' : t('page.words', { count: words })
  ]
    .filter((item) => item.length > 0)
    .join(' · ');

  return (
    <section className={styles.readPage}>
      <div className={styles.chrome}>
        <IconButton
          label={t('page.back')}
          variant="outlined"
          className={styles.float}
          onClick={() => back(entry.memberId)}
        >
          <CaretLeftIcon weight="regular" />
        </IconButton>
        <IconButton label={t('page.edit')} variant="outlined" className={styles.float} onClick={() => setEditing(true)}>
          <PencilSimpleIcon weight="regular" />
        </IconButton>
      </div>
      <div className={styles.read}>
        <h1 className={styles.readTitle}>{entry.title}</h1>
        <p className={styles.byline}>{byline}</p>
        {entry.locked ? (
          <p className={styles.note}>{t('page.sealed')}</p>
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
