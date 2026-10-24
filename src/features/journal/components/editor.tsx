import { useMutation, useQueryClient } from '@tanstack/react-query';
import clsx from 'clsx';
import { useRef, useState } from 'react';
import type { JournalEntry, JournalEntryInput, JournalKind } from '@/bridge/journal';
import { journalEntryCreate, journalEntryDelete, journalEntryUpdate } from '@/bridge/journal';
import { Button } from '@/components/button';
import { Checkbox } from '@/components/checkbox';
import { DateField } from '@/components/date-field';
import type { MarkdownEditorHandle } from '@/components/markdown';
import { MarkdownField, retainBodyImages } from '@/components/markdown';
import styles from '@/components/record.module.css';
import { Select } from '@/components/select';
import { TextField } from '@/components/text-field';
import { refreshJournal } from '@/query/journal/query';
import { fromDateInputValue, startOfLocalDay, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';

const kindLabel: Record<JournalKind, string> = {
  diary: '日记',
  spark: '灵感',
  writing: '写作'
};

type JournalEditorProps = {
  entry: JournalEntry | null;
  kind: JournalKind;
  className?: string;
  onClose?: () => void;
  onSaved: (entry: JournalEntry) => void;
  onDeleted?: () => void;
};

export function JournalEditor({ entry, kind, className, onClose, onSaved, onDeleted }: JournalEditorProps) {
  const queryClient = useQueryClient();
  const markdownRef = useRef<MarkdownEditorHandle>(null);
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
    mutationFn: async () => {
      const next = input();
      const saved = entry ? await journalEntryUpdate(entry.id, next) : await journalEntryCreate(next);
      if (entry && !entry.locked) {
        await retainBodyImages('journal_entry', entry.id, next.body);
        markdownRef.current?.saved();
      }
      return saved;
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
      className={clsx(styles.stack, className)}
      onSubmit={(event) => {
        event.preventDefault();
        save.mutate();
      }}
    >
      <div className={styles.pair}>
        <Select label="类型" value={draftKind} onChange={(event) => setDraftKind(event.target.value as JournalKind)}>
          {(Object.keys(kindLabel) as JournalKind[]).map((item) => (
            <option key={item} value={item}>
              {kindLabel[item]}
            </option>
          ))}
        </Select>
        <DateField label="发生时间" value={occurredAt} onChange={setOccurredAt} />
      </div>
      <TextField label="标题" value={title} required onChange={(event) => setTitle(event.target.value)} />
      <MarkdownField
        ref={markdownRef}
        label="正文"
        hint={entry?.locked && body === '' ? '正文已封存。解开之后才能改。' : undefined}
        value={body}
        disabled={Boolean(entry?.locked)}
        image={entry?.locked ? undefined : entry ? { owner: 'journal_entry', ownerId: entry.id } : null}
        onChange={setBody}
      />
      <Checkbox label="标为高光" checked={highlight} onChange={(event) => setHighlight(event.target.checked)} />
      {save.error ? <p className={styles.error}>{errorMessage(save.error)}</p> : null}
      {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
      <div className={styles.actions}>
        {entry && onDeleted ? (
          <Button
            variant="outlined"
            tone="destructive"
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
          </Button>
        ) : null}
        <span className={styles.spacer} />
        {onClose ? (
          <Button variant="outlined" disabled={pending} onClick={onClose}>
            取消
          </Button>
        ) : null}
        <Button type="submit" className={styles.trailing} disabled={pending}>
          保存
        </Button>
      </div>
    </form>
  );
}
