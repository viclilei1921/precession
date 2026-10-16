import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import type { GrowthEntry, GrowthKind } from '@/bridge/growth';
import { growthEntryCreate, growthEntryDelete, growthEntryUpdate } from '@/bridge/growth';
import { DateField } from '@/components/date-field';
import styles from '@/components/record.module.css';
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
  const [draftKind, setDraftKind] = useState<GrowthKind>(entry?.kind ?? kind);
  const [title, setTitle] = useState(entry?.title ?? '');
  const [body, setBody] = useState(entry?.body ?? '');
  const [occurredAt, setOccurredAt] = useState(toDateInputValue(entry?.occurredAt ?? startOfLocalDay()));
  const [highlight, setHighlight] = useState(entry?.highlight ?? false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const save = useMutation({
    mutationFn: () => {
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
      return entry ? growthEntryUpdate(entry.id, input) : growthEntryCreate(input);
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
        <label className={styles.field}>
          类型
          <select value={draftKind} onChange={(event) => setDraftKind(event.target.value as GrowthKind)}>
            <option value="milestone">里程碑</option>
            <option value="moment">瞬间</option>
          </select>
        </label>
        <DateField label="发生时间" value={occurredAt} onChange={setOccurredAt} />
      </div>
      <label className={styles.field}>
        标题
        <input value={title} required onChange={(event) => setTitle(event.target.value)} />
      </label>
      <label className={styles.field}>
        说明
        <textarea value={body} onChange={(event) => setBody(event.target.value)} />
      </label>
      <label className={styles.check}>
        <input type="checkbox" checked={highlight} onChange={(event) => setHighlight(event.target.checked)} />
        标为高光，进入年度之书
      </label>
      {save.error ? <p className={styles.error}>{errorMessage(save.error)}</p> : null}
      {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
      <div className={styles.actions}>
        {entry ? (
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
        <button type="button" className={styles.ghost} disabled={pending} onClick={onClose}>
          取消
        </button>
        <button type="submit" className={styles.primary} disabled={pending}>
          保存
        </button>
      </div>
    </form>
  );
}
