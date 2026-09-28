import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate, useParams } from '@tanstack/react-router';
import { useState } from 'react';
import type { JournalEntry } from '@/bridge/journal';
import {
  journalCitationCreate,
  journalCitationDelete,
  journalCitationList,
  journalEntryCreate,
  journalEntryOpen,
  journalEntrySeal,
  journalEntryUpdate,
  journalLinkCreate,
  journalLinkList
} from '@/bridge/journal';
import { bookNoteGet, bookNoteList } from '@/bridge/library';
import { PATH } from '@/router/path';
import { journalEntryQuery, refreshJournal } from '@/shared/data/journal/query';
import { bookListQuery } from '@/shared/data/library/query';
import { errorMessage } from '@/shared/lib/error';
import styles from '@/shared/ui/record.module.css';
import { JournalEditor } from './editor';

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
        <button type="button" className={styles.ghost} onClick={() => void navigate({ to: PATH.journal })}>
          返回列表
        </button>
      </header>
      <div className={styles.card}>
        <JournalEditor
          key={entryQuery.data.updatedAt}
          entry={entryQuery.data}
          kind={entryQuery.data.kind}
          onSaved={() => undefined}
          onDeleted={() => void navigate({ to: PATH.journal })}
        />
      </div>
      <Promote entry={entryQuery.data} />
      {entryQuery.data.kind === 'diary' ? <Seal entry={entryQuery.data} /> : null}
      <Citations entryId={entryQuery.data.id} />
    </section>
  );
}

function Promote({ entry }: { entry: JournalEntry }) {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const linksQuery = useQuery({
    queryKey: ['journal', 'links', entry.id],
    queryFn: () => journalLinkList(entry.id)
  });
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

function Seal({ entry }: { entry: JournalEntry }) {
  const queryClient = useQueryClient();
  const [password, setPassword] = useState('');
  const [plain, setPlain] = useState('');
  const seal = useMutation({
    mutationFn: async () => {
      const body = await journalEntrySeal(plain || entry.body, password);
      return journalEntryUpdate(entry.id, {
        kind: entry.kind,
        occurredAt: entry.occurredAt,
        title: entry.title,
        body,
        locked: true,
        highlight: entry.highlight,
        memberIds: entry.memberIds,
        tagIds: entry.tagIds,
        placeIds: entry.placeIds
      });
    },
    onSuccess: async () => {
      setPassword('');
      setPlain('');
      await refreshJournal(queryClient);
    }
  });
  const open = useMutation({
    mutationFn: () => journalEntryOpen(entry.body, password),
    onSuccess: async (text) => {
      setPlain(text);
      await journalEntryUpdate(entry.id, {
        kind: entry.kind,
        occurredAt: entry.occurredAt,
        title: entry.title,
        body: text,
        locked: false,
        highlight: entry.highlight,
        memberIds: entry.memberIds,
        tagIds: entry.tagIds,
        placeIds: entry.placeIds
      });
      await refreshJournal(queryClient);
    }
  });

  return (
    <div className={styles.card}>
      <h2 className={styles.cardTitle}>正文封存</h2>
      <p className={styles.note}>{entry.locked ? '这篇日记的正文已经单独封存。' : '可以用另一套密码把正文封起来。'}</p>
      {plain ? <p className={styles.note}>{plain}</p> : null}
      <label className={styles.field}>
        正文密码
        <input type="password" value={password} onChange={(event) => setPassword(event.target.value)} />
      </label>
      <div className={styles.actions}>
        {entry.locked ? (
          <button
            type="button"
            className={styles.primary}
            disabled={open.isPending || password.length === 0}
            onClick={() => open.mutate()}
          >
            解开
          </button>
        ) : (
          <button
            type="button"
            className={styles.primary}
            disabled={seal.isPending || password.length === 0}
            onClick={() => seal.mutate()}
          >
            封存
          </button>
        )}
      </div>
      {seal.error ? <p className={styles.error}>{errorMessage(seal.error)}</p> : null}
      {open.error ? <p className={styles.error}>{errorMessage(open.error)}</p> : null}
    </div>
  );
}

function Citations({ entryId }: { entryId: string }) {
  const queryClient = useQueryClient();
  const booksQuery = useQuery(bookListQuery);
  const citationsQuery = useQuery({
    queryKey: ['journal', 'citations', entryId],
    queryFn: () => journalCitationList(entryId)
  });
  const [bookId, setBookId] = useState('');
  const notesQuery = useQuery({
    queryKey: ['library', 'notes', bookId],
    queryFn: () => bookNoteList(bookId),
    enabled: bookId.length > 0
  });
  const [noteId, setNoteId] = useState('');
  const add = useMutation({
    mutationFn: () => journalCitationCreate(entryId, noteId),
    onSuccess: async () => {
      setNoteId('');
      await queryClient.invalidateQueries({ queryKey: ['journal', 'citations', entryId] });
    }
  });
  const remove = useMutation({
    mutationFn: (id: string) => journalCitationDelete(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['journal', 'citations', entryId] });
    }
  });
  const books = booksQuery.data ?? [];

  return (
    <div className={styles.card}>
      <h2 className={styles.cardTitle}>引用书摘</h2>
      {(citationsQuery.data ?? []).length === 0 ? <p className={styles.empty}>还没有引用。</p> : null}
      {(citationsQuery.data ?? []).map((citation) => (
        <CitationRow key={citation.id} noteId={citation.bookNoteId} onRemove={() => remove.mutate(citation.id)} />
      ))}
      <div className={styles.pair}>
        <label className={styles.field}>
          书
          <select
            value={bookId}
            onChange={(event) => {
              setBookId(event.target.value);
              setNoteId('');
            }}
          >
            <option value="">选择一本书</option>
            {books.map((book) => (
              <option key={book.id} value={book.id}>
                {book.title}
              </option>
            ))}
          </select>
        </label>
        <label className={styles.field}>
          书摘或笔记
          <select value={noteId} onChange={(event) => setNoteId(event.target.value)}>
            <option value="">选择一条</option>
            {(notesQuery.data ?? []).map((note) => (
              <option key={note.id} value={note.id}>
                {note.title}
              </option>
            ))}
          </select>
        </label>
      </div>
      <button
        type="button"
        className={styles.primary}
        disabled={noteId.length === 0 || add.isPending}
        onClick={() => add.mutate()}
      >
        引用
      </button>
      {add.error ? <p className={styles.error}>{errorMessage(add.error)}</p> : null}
      {books.length === 0 ? <p className={styles.note}>书库里还没有书，先去添加。</p> : null}
    </div>
  );
}

function CitationRow({ noteId, onRemove }: { noteId: string; onRemove: () => void }) {
  const noteQuery = useQuery({
    queryKey: ['library', 'note', noteId],
    queryFn: () => bookNoteGet(noteId)
  });

  return (
    <div className={styles.row}>
      <span className={styles.rowTitle}>{noteQuery.data?.title ?? '书摘'}</span>
      <button type="button" className={styles.ghost} onClick={onRemove}>
        取消
      </button>
    </div>
  );
}
