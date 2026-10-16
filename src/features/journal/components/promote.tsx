import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import type { JournalEntry } from '@/bridge/journal';
import { journalEntryCreate, journalLinkCreate } from '@/bridge/journal';
import styles from '@/components/record.module.css';
import { journalLinkListQuery, refreshJournal } from '@/query/journal/query';
import { PATH } from '@/router/path';
import { errorMessage } from '@/utils/error';

/** 把手记流转到下一种类型。 */
export function Promote({ entry }: { entry: JournalEntry }) {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const linksQuery = useQuery(journalLinkListQuery(entry.id));
  const promote = useMutation({
    mutationFn: async () => {
      const nextKind = entry.kind === 'spark' ? 'writing' : 'diary';
      const linkKind = entry.kind === 'spark' ? 'spark_to_writing' : 'writing_to_diary';
      const created = await journalEntryCreate({
        kind: nextKind,
        occurredAt: Date.now(),
        title: entry.title,
        body: entry.locked ? '' : entry.body,
        locked: false,
        highlight: false,
        memberIds: entry.memberIds,
        tagIds: entry.tagIds,
        placeIds: entry.placeIds
      });
      await journalLinkCreate(entry.id, created.id, linkKind);
      return created;
    },
    onSuccess: async (created) => {
      await refreshJournal(queryClient);
      void navigate({ to: PATH.journalEntry, params: { entryId: created.id } });
    }
  });

  return (
    <div className={styles.card}>
      <h2 className={styles.cardTitle}>流转</h2>
      {(linksQuery.data ?? []).map((link) => {
        const otherId = link.fromId === entry.id ? link.toId : link.fromId;
        return (
          <button
            key={link.id}
            type="button"
            className={styles.row}
            onClick={() => void navigate({ to: PATH.journalEntry, params: { entryId: otherId } })}
          >
            <span className={styles.rowTitle}>{link.kind === 'spark_to_writing' ? '灵感到写作' : '写作到日记'}</span>
            <span className={styles.meta}>打开关联</span>
          </button>
        );
      })}
      {entry.kind === 'writing' || entry.kind === 'spark' ? (
        <button type="button" className={styles.primary} disabled={promote.isPending} onClick={() => promote.mutate()}>
          {entry.kind === 'spark' ? '扩写成写作' : '归入日记'}
        </button>
      ) : (
        <p className={styles.note}>日记是这条流转的尽头。</p>
      )}
      {promote.error ? <p className={styles.error}>{errorMessage(promote.error)}</p> : null}
    </div>
  );
}
