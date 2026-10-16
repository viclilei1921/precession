import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import type { BookNote, BookNoteKind } from '@/bridge/library';
import { bookNoteCreate, bookNoteDelete, bookNoteUpdate } from '@/bridge/library';
import { DateField } from '@/components/date-field';
import styles from '@/components/record.module.css';
import { refreshLibrary } from '@/query/library/query';
import { fromDateInputValue, startOfLocalDay, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';

export function NoteForm({ bookId, note, onClose }: { bookId: string; note: BookNote | null; onClose: () => void }) {
  const queryClient = useQueryClient();
  const [kind, setKind] = useState<BookNoteKind>(note?.kind ?? 'excerpt');
  const [title, setTitle] = useState(note?.title ?? '');
  const [body, setBody] = useState(note?.body ?? '');
  const [chapter, setChapter] = useState(note?.chapter ?? '');
  const [location, setLocation] = useState(note?.location ?? '');
  const [occurredAt, setOccurredAt] = useState(toDateInputValue(note?.occurredAt ?? startOfLocalDay()));
  const [highlight, setHighlight] = useState(note?.highlight ?? false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const save = useMutation({
    mutationFn: () => {
      const input = {
        bookId,
        kind,
        occurredAt: fromDateInputValue(occurredAt) ?? startOfLocalDay(),
        title,
        body,
        chapter,
        location,
        locked: note?.locked ?? false,
        highlight,
        memberIds: note?.memberIds ?? [],
        tagIds: note?.tagIds ?? [],
        placeIds: note?.placeIds ?? []
      };
      return note ? bookNoteUpdate(note.id, input) : bookNoteCreate(input);
    },
    onSuccess: async () => {
      await refreshLibrary(queryClient);
      onClose();
    }
  });
  const remove = useMutation({
    mutationFn: () => (note ? bookNoteDelete(note.id) : Promise.reject(new Error('记录不存在'))),
    onSuccess: async () => {
      await refreshLibrary(queryClient);
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
          <select value={kind} onChange={(event) => setKind(event.target.value as BookNoteKind)}>
            <option value="excerpt">书摘</option>
            <option value="note">笔记</option>
          </select>
        </label>
        <DateField label="时间" value={occurredAt} onChange={setOccurredAt} />
      </div>
      <label className={styles.field}>
        标题
        <input value={title} required onChange={(event) => setTitle(event.target.value)} />
      </label>
      <label className={styles.field}>
        正文
        <textarea value={body} onChange={(event) => setBody(event.target.value)} />
      </label>
      <div className={styles.pair}>
        <label className={styles.field}>
          章节
          <input value={chapter} onChange={(event) => setChapter(event.target.value)} />
        </label>
        <label className={styles.field}>
          位置
          <input value={location} onChange={(event) => setLocation(event.target.value)} />
        </label>
      </div>
      <label className={styles.check}>
        <input type="checkbox" checked={highlight} onChange={(event) => setHighlight(event.target.checked)} />
        标为高光
      </label>
      {save.error ? <p className={styles.error}>{errorMessage(save.error)}</p> : null}
      {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
      <div className={styles.actions}>
        {note ? (
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
