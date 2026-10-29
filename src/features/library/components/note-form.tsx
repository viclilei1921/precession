import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useRef, useState } from 'react';
import type { BookNote, BookNoteKind } from '@/bridge/library';
import { bookNoteCreate, bookNoteDelete, bookNoteUpdate } from '@/bridge/library';
import { Button } from '@/components/button';
import { SegmentButton } from '@/components/button/segment';
import { Checkbox } from '@/components/checkbox';
import { DateField } from '@/components/date-field';
import type { MarkdownEditorHandle } from '@/components/markdown';
import {
  looseImageOwner,
  MarkdownField,
  markdownPlain,
  mountLooseImages,
  retainBodyImages
} from '@/components/markdown';
import recordStyles from '@/components/record.module.css';
import { Scroll } from '@/components/scroll';
import { TextField } from '@/components/text-field';
import styles from '@/features/plan/drawer.module.css';
import { refreshFiles } from '@/query/files/query';
import { refreshLibrary } from '@/query/library/query';
import { fromDateInputValue, startOfLocalDay, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';

/** 列表和卡片上看到的句子。锁定的不露出正文 */
export function noteBody(note: { body: string; title: string; locked: boolean }) {
  if (note.locked) {
    return '这条已锁定。';
  }
  return markdownPlain(note.body) || note.title.trim();
}

type NoteFormProps = {
  bookId: string;
  note: BookNote | null;
  preset?: { kind?: BookNoteKind; chapter?: string; body?: string };
  /** 阅读里的书摘模式只记书摘 */
  lockKind?: boolean;
  onClose: () => void;
};

export function NoteForm({ bookId, note, preset, lockKind, onClose }: NoteFormProps) {
  const queryClient = useQueryClient();
  const markdownRef = useRef<MarkdownEditorHandle>(null);
  const [kind, setKind] = useState<BookNoteKind>(lockKind ? 'excerpt' : (note?.kind ?? preset?.kind ?? 'excerpt'));
  const [body, setBody] = useState(
    note?.locked ? '' : note?.body.trim() ? note.body : (preset?.body ?? note?.title ?? '')
  );
  const [chapter, setChapter] = useState(note?.chapter ?? preset?.chapter ?? '');
  const [location, setLocation] = useState(note?.location ?? '');
  const [occurredAt, setOccurredAt] = useState(toDateInputValue(note?.occurredAt ?? startOfLocalDay()));
  const [highlight, setHighlight] = useState(note?.highlight ?? false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [empty, setEmpty] = useState(false);
  const save = useMutation({
    mutationFn: async () => {
      const sealed = Boolean(note?.locked) && body.trim() === '';
      const text = sealed ? note?.body.trim() || note?.title.trim() || '已锁定' : body.trim();
      const first = text.split('\n')[0] || text;
      const input = {
        bookId,
        kind: lockKind ? 'excerpt' : kind,
        occurredAt: fromDateInputValue(occurredAt) ?? startOfLocalDay(),
        title: markdownPlain(first) || first,
        body: sealed ? (note?.body ?? '') : body.trim(),
        chapter,
        location,
        locked: sealed,
        highlight,
        memberIds: note?.memberIds ?? [],
        tagIds: note?.tagIds ?? [],
        placeIds: note?.placeIds ?? []
      };
      const saved = note ? await bookNoteUpdate(note.id, input) : await bookNoteCreate(input);
      if (note && !note.locked) {
        await retainBodyImages('book_note', note.id, input.body);
        markdownRef.current?.saved(input.body);
      } else if (!note) {
        try {
          await mountLooseImages('book_note', saved.id, input.body);
        } catch (error) {
          await bookNoteDelete(saved.id).catch(() => undefined);
          throw error;
        }
        markdownRef.current?.saved(input.body);
      }
      return saved;
    },
    onSuccess: async () => {
      await refreshLibrary(queryClient);
      await refreshFiles(queryClient);
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
      className={styles.form}
      onSubmit={(event) => {
        event.preventDefault();
        if (!body.trim() && !note?.locked) {
          setEmpty(true);
          return;
        }
        setEmpty(false);
        save.mutate();
      }}
    >
      <Scroll className={styles.sheet} axis="vertical">
        <div className={styles.main}>
          {lockKind ? (
            <p className={recordStyles.note}>写下想留下的句子。章节先填了当前这一章。</p>
          ) : (
            <div className={recordStyles.segments}>
              <SegmentButton selected={kind === 'excerpt'} onClick={() => setKind('excerpt')}>
                书摘
              </SegmentButton>
              <SegmentButton selected={kind === 'note'} onClick={() => setKind('note')}>
                笔记
              </SegmentButton>
            </div>
          )}
          <MarkdownField
            ref={markdownRef}
            label="正文"
            hint={note?.locked && body === '' ? '这条已锁定。' : undefined}
            value={body}
            disabled={Boolean(note?.locked)}
            image={note?.locked ? undefined : note ? { owner: 'book_note', ownerId: note.id } : looseImageOwner}
            onChange={setBody}
          />
          <div className={recordStyles.pair}>
            <TextField label="章节" value={chapter} onChange={(event) => setChapter(event.target.value)} />
            <TextField label="位置" value={location} onChange={(event) => setLocation(event.target.value)} />
          </div>
          <DateField label="时间" value={occurredAt} onChange={setOccurredAt} />
          <Checkbox label="标为高光" checked={highlight} onChange={(event) => setHighlight(event.target.checked)} />
        </div>
      </Scroll>
      <div className={styles.foot}>
        {empty ? <p className={styles.error}>先写下内容</p> : null}
        {save.error ? <p className={styles.error}>{errorMessage(save.error)}</p> : null}
        {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
        <div className={recordStyles.actions}>
          {note ? (
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
          <span className={recordStyles.spacer} />
          <Button variant="outlined" disabled={pending} onClick={onClose}>
            取消
          </Button>
          <Button type="submit" disabled={pending}>
            保存
          </Button>
        </div>
      </div>
    </form>
  );
}
