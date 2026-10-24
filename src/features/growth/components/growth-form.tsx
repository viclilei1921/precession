import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useRef, useState } from 'react';
import type { GrowthEntry, GrowthKind } from '@/bridge/growth';
import { growthEntryCreate, growthEntryDelete, growthEntryUpdate } from '@/bridge/growth';
import { Button } from '@/components/button';
import { Checkbox } from '@/components/checkbox';
import { DateField } from '@/components/date-field';
import type { MarkdownEditorHandle } from '@/components/markdown';
import { MarkdownField, retainBodyImages } from '@/components/markdown';
import styles from '@/components/record.module.css';
import { Select } from '@/components/select';
import { TextField } from '@/components/text-field';
import { refreshGrowth } from '@/query/growth/query';
import { fromDateInputValue, startOfLocalDay, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';

export function GrowthForm({
  memberId,
  entry,
  kind,
  onClose
}: {
  memberId: string;
  entry: GrowthEntry | null;
  kind: GrowthKind;
  onClose: () => void;
}) {
  const queryClient = useQueryClient();
  const markdownRef = useRef<MarkdownEditorHandle>(null);
  const [draftKind, setDraftKind] = useState<GrowthKind>(entry?.kind ?? kind);
  const [title, setTitle] = useState(entry?.title ?? '');
  const [body, setBody] = useState(entry?.body ?? '');
  const [occurredAt, setOccurredAt] = useState(toDateInputValue(entry?.occurredAt ?? startOfLocalDay()));
  const [highlight, setHighlight] = useState(entry?.highlight ?? false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const save = useMutation({
    mutationFn: async () => {
      const input = {
        kind: draftKind,
        memberId,
        occurredAt: fromDateInputValue(occurredAt) ?? startOfLocalDay(),
        title,
        body,
        locked: entry?.locked ?? false,
        highlight,
        memberIds: entry?.memberIds ?? [],
        tagIds: entry?.tagIds ?? [],
        placeIds: entry?.placeIds ?? []
      };
      const saved = entry ? await growthEntryUpdate(entry.id, input) : await growthEntryCreate(input);
      if (entry) {
        await retainBodyImages('growth_entry', entry.id, input.body);
        markdownRef.current?.saved();
      }
      return saved;
    },
    onSuccess: async () => {
      await refreshGrowth(queryClient);
      onClose();
    }
  });
  const remove = useMutation({
    mutationFn: () => (entry ? growthEntryDelete(entry.id) : Promise.reject(new Error('记录不存在'))),
    onSuccess: async () => {
      await refreshGrowth(queryClient);
      onClose();
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
        <Select label="类型" value={draftKind} onChange={(event) => setDraftKind(event.target.value as GrowthKind)}>
          <option value="milestone">里程碑</option>
          <option value="moment">瞬间</option>
        </Select>
        <DateField label="发生时间" value={occurredAt} onChange={setOccurredAt} />
      </div>
      <TextField label="标题" value={title} required onChange={(event) => setTitle(event.target.value)} />
      <MarkdownField
        ref={markdownRef}
        label="说明"
        value={body}
        image={entry ? { owner: 'growth_entry', ownerId: entry.id } : null}
        onChange={setBody}
      />
      <Checkbox
        label="标为高光，进入年度之书"
        checked={highlight}
        onChange={(event) => setHighlight(event.target.checked)}
      />
      {save.error ? <p className={styles.error}>{errorMessage(save.error)}</p> : null}
      {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
      <div className={styles.actions}>
        {entry ? (
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
        <Button variant="outlined" disabled={pending} onClick={onClose}>
          取消
        </Button>
        <Button type="submit" className={styles.trailing} disabled={pending}>
          保存
        </Button>
      </div>
    </form>
  );
}
