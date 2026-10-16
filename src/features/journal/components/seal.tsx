import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import type { JournalEntry } from '@/bridge/journal';
import { journalEntryOpen, journalEntrySeal, journalEntryUpdate } from '@/bridge/journal';
import { PasswordInput } from '@/components/password-input';
import styles from '@/components/record.module.css';
import { refreshJournal } from '@/query/journal/query';
import { errorMessage } from '@/utils/error';

/** 用单独密码封存或解开日记正文。 */
export function Seal({ entry }: { entry: JournalEntry }) {
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
      <label className={styles.field} htmlFor="journal-body-password">
        正文密码
        <PasswordInput
          id="journal-body-password"
          inputClassName={styles.passwordInput}
          value={password}
          onChange={(event) => setPassword(event.target.value)}
        />
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
