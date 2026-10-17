import { CaretLeftIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useParams } from '@tanstack/react-router';
import { Button } from '@/components/button';
import styles from '@/components/record.module.css';
import { journalEntryQuery } from '@/query/journal/query';
import { PATH } from '@/router/path';
import { errorMessage } from '@/utils/error';
import { Citations } from './components/citations';
import { JournalEditor } from './components/editor';
import { Promote } from './components/promote';
import { Seal } from './components/seal';

export function JournalEntryPage() {
  const { entryId } = useParams({ from: '/journal/$entryId' });
  const navigate = useNavigate();
  const entryQuery = useQuery(journalEntryQuery(entryId));

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

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>编辑手记</h1>
        <Button variant="outlined" onClick={() => void navigate({ to: PATH.journal })}>
          <CaretLeftIcon weight="regular" />
          返回列表
        </Button>
      </header>
      <JournalEditor
        key={entryQuery.data.updatedAt}
        className={styles.card}
        entry={entryQuery.data}
        kind={entryQuery.data.kind}
        onSaved={() => undefined}
        onDeleted={() => void navigate({ to: PATH.journal })}
      />
      <Promote entry={entryQuery.data} />
      {entryQuery.data.kind === 'diary' ? <Seal entry={entryQuery.data} /> : null}
      <Citations entryId={entryQuery.data.id} />
    </section>
  );
}
