import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import type { JournalEntry, JournalEntryInput, JournalKind } from '@/bridge/journal';
import { journalEntryCreate, journalEntryDelete, journalEntryUpdate } from '@/bridge/journal';
import { refreshJournal } from '@/shared/data/journal/query';
import { fromDateInputValue, startOfLocalDay, toDateInputValue } from '@/shared/lib/day';
import { errorMessage } from '@/shared/lib/error';
import styles from '@/shared/ui/record.module.css';

const kindLabel: Record<JournalKind, string> = {
  diary: '日记',
  spark: '灵感',
  writing: '写作'
};

type JournalEditorProps = {
  entry: JournalEntry | null;
  kind: JournalKind;
  onClose?: () => void;
  onSaved: (entry: JournalEntry) => void;
  onDeleted?: () => void;
};

export function JournalEditor({ entry, kind, onClose, onSaved, onDeleted }: JournalEditorProps) {
  const queryClient = useQueryClient();
  const [draftKind, setDraftKind] = useState<JournalKind>(entry?.kind ?? kind);
  const [title, setTitle] = useState(entry?.title ?? '');
  const [body, setBody] = useState(entry?.locked ? '' : (entry?.body ?? ''));
  const [occurredAt, setOccurredAt] = useState(toDateInputValue(entry?.occurredAt ?? startOfLocalDay()));
  const [highlight, setHighlight] = useState(entry?.highlight ?? false);
  const [confirmDelete, setConfirmDelete] = useState(false);

  function input(): JournalEntryInput {
    const stillSealed = Boolean(entry?.locked) && body.trim() === '';
    return {
      kind: draftKind,
      occurredAt: fromDateInputValue(occurredAt) ?? startOfLocalDay(),
      title,
      body: stillSealed ? (entry?.body ?? '') : body,
      locked: stillSealed,
      highlight,
      memberIds: entry?.memberIds ?? [],
      tagIds: entry?.tagIds ?? [],
      placeIds: entry?.placeIds ?? []
    };
  }

  const save = useMutation({
    mutationFn: () => {
      const next = input();
      return entry ? journalEntryUpdate(entry.id, next) : journalEntryCreate(next);
    },
    onSuccess: async (saved) => {
      await refreshJournal(queryClient);
      onSaved(saved);
    }
  });
  const remove = useMutation({
    mutationFn: () => {
      if (!entry) {
        return Promise.reject(new Error('手记不存在'));
      }
      return journalEntryDelete(entry.id);
    },
    onSuccess: async () => {
      await refreshJournal(queryClient);
      onDeleted?.();
    }
  });
  const pending = save.isPending || remove.isPending;

  return (
    <form
      className={styles.stack}
      onSubmit={(event) => {
        event.preventDefault();
        save.mutate();
      }}
    >
      <div className={styles.pair}>
        <label className={styles.field}>
          类型
          <select value={draftKind} onChange={(event) => setDraftKind(event.target.value as JournalKind)}>
            {(Object.keys(kindLabel) as JournalKind[]).map((item) => (
              <option key={item} value={item}>
                {kindLabel[item]}
              </option>
            ))}
          </select>
        </label>
        <label className={styles.field}>
          发生时间
          <input type="date" value={occurredAt} onChange={(event) => setOccurredAt(event.target.value)} />
        </label>
      </div>
      <label className={styles.field}>
        标题
        <input value={title} required onChange={(event) => setTitle(event.target.value)} />
      </label>
      <label className={styles.field}>
        正文
        {entry?.locked && body === '' ? <span className={styles.note}>正文已封存。解开之后才能改。</span> : null}
        <textarea value={body} onChange={(event) => setBody(event.target.value)} />
      </label>
      <label className={styles.check}>
        <input type="checkbox" checked={highlight} onChange={(event) => setHighlight(event.target.checked)} />
        标为高光
      </label>
      {save.error ? <p className={styles.error}>{errorMessage(save.error)}</p> : null}
      {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
      <div className={styles.actions}>
        {entry && onDeleted ? (
          <button
            type="button"
            className={styles.danger}
            disabled={pending}
            onClick={() => {
              if (!confirmDelete) {
                setConfirmDelete(true);
                return;
              }
              remove.mutate();
            }}
          >
            {confirmDelete ? '确认删除' : '删除'}
          </button>
        ) : null}
        <span className={styles.spacer} />
        {onClose ? (
          <button type="button" className={styles.ghost} disabled={pending} onClick={onClose}>
            取消
          </button>
        ) : null}
        <button type="submit" className={styles.primary} disabled={pending}>
          保存
        </button>
      </div>
    </form>
  );
}
