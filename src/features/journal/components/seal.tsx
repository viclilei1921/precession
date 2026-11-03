import { LockIcon, LockOpenIcon } from '@phosphor-icons/react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { JournalEntry } from '@/bridge/journal';
import { journalEntryOpen, journalEntrySeal, journalEntryUpdate } from '@/bridge/journal';
import { IconButton } from '@/components/button/icon';
import { retainBodyImages } from '@/components/markdown';
import { PasswordInput } from '@/components/password-input';
import styles from '@/components/record.module.css';
import { refreshJournal } from '@/query/journal/query';
import { errorMessage } from '@/utils/error';

/** 用单独密码封存或解开日记正文。`readBody` 读编辑器里的当前正文。 */
export function Seal({ entry, readBody, onKept }: { entry: JournalEntry; readBody: () => string; onKept: () => void }) {
  const { t } = useTranslation('journal');
  const queryClient = useQueryClient();
  const [password, setPassword] = useState('');
  const seal = useMutation({
    mutationFn: async () => {
      const plain = readBody();
      const body = await journalEntrySeal(plain, password);
      await journalEntryUpdate(entry.id, {
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
      try {
        await retainBodyImages('journal_entry', entry.id, plain);
      } finally {
        onKept();
      }
    },
    onSuccess: async () => {
      setPassword('');
      await refreshJournal(queryClient);
    }
  });
  const open = useMutation({
    mutationFn: () => journalEntryOpen(entry.body, password),
    onSuccess: async (text) => {
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
    <div className={styles.stack}>
      <PasswordInput
        id="journal-body-password"
        label={t('seal.password')}
        value={password}
        onChange={(event) => setPassword(event.target.value)}
        onKeyDown={(event) => {
          if (event.key !== 'Enter') {
            return;
          }
          event.preventDefault();
          if (password.length === 0) {
            return;
          }
          if (entry.locked) {
            open.mutate();
            return;
          }
          seal.mutate();
        }}
      />
      <div className={styles.actions}>
        {entry.locked ? (
          <IconButton
            label={t('seal.unseal')}
            disabled={open.isPending || password.length === 0}
            onClick={() => open.mutate()}
          >
            <LockOpenIcon weight="regular" />
          </IconButton>
        ) : (
          <IconButton
            label={t('seal.seal')}
            disabled={seal.isPending || password.length === 0}
            onClick={() => seal.mutate()}
          >
            <LockIcon weight="regular" />
          </IconButton>
        )}
      </div>
      {seal.error ? <p className={styles.error}>{errorMessage(seal.error)}</p> : null}
      {open.error ? <p className={styles.error}>{errorMessage(open.error)}</p> : null}
    </div>
  );
}
