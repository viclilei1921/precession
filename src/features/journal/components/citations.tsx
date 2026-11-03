import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { journalCitationCreate, journalCitationDelete } from '@/bridge/journal';
import { Button } from '@/components/button';
import styles from '@/components/record.module.css';
import { Select } from '@/components/select';
import { displayTitle } from '@/features/library/book';
import { noteBody } from '@/features/library/components/note-form';
import { journalCitationListQuery, refreshJournalCitations } from '@/query/journal/query';
import { bookListQuery, bookNoteListQuery, bookNoteQuery } from '@/query/library/query';
import { errorMessage } from '@/utils/error';

type CitationsProps = {
  entryId: string;
  /** 为真时才露出选书和选摘 */
  adding: boolean;
  onAdded?: () => void;
};

/** 给一条手记引用书摘或读书笔记。平时只列出已引用的。 */
export function Citations({ entryId, adding, onAdded }: CitationsProps) {
  const { t } = useTranslation(['journal', 'common']);
  const queryClient = useQueryClient();
  const booksQuery = useQuery(bookListQuery);
  const citationsQuery = useQuery(journalCitationListQuery(entryId));
  const [bookId, setBookId] = useState('');
  const notesQuery = useQuery({
    ...bookNoteListQuery(bookId),
    enabled: bookId.length > 0
  });
  const [noteId, setNoteId] = useState('');
  const add = useMutation({
    mutationFn: () => journalCitationCreate(entryId, noteId),
    onSuccess: async () => {
      setNoteId('');
      setBookId('');
      await refreshJournalCitations(queryClient, entryId);
      onAdded?.();
    }
  });
  const remove = useMutation({
    mutationFn: (id: string) => journalCitationDelete(id),
    onSuccess: async () => {
      await refreshJournalCitations(queryClient, entryId);
    }
  });
  const books = booksQuery.data ?? [];

  return (
    <div className={styles.stack}>
      {(citationsQuery.data ?? []).length === 0 ? <p className={styles.empty}>{t('citations.empty')}</p> : null}
      {(citationsQuery.data ?? []).map((citation) => (
        <CitationRow key={citation.id} noteId={citation.bookNoteId} onRemove={() => remove.mutate(citation.id)} />
      ))}
      {adding ? (
        <>
          <Select
            label={t('citations.book')}
            value={bookId}
            onChange={(event) => {
              setBookId(event.target.value);
              setNoteId('');
            }}
          >
            <option value="">{t('citations.selectBook')}</option>
            {books.map((book) => (
              <option key={book.id} value={book.id}>
                {displayTitle(book.title)}
              </option>
            ))}
          </Select>
          <Select label={t('citations.note')} value={noteId} onChange={(event) => setNoteId(event.target.value)}>
            <option value="">{t('citations.selectNote')}</option>
            {(notesQuery.data ?? []).map((note) => (
              <option key={note.id} value={note.id}>
                {noteBody(note)}
              </option>
            ))}
          </Select>
          <Button disabled={noteId.length === 0 || add.isPending} onClick={() => add.mutate()}>
            {t('citations.add')}
          </Button>
          {add.error ? <p className={styles.error}>{errorMessage(add.error)}</p> : null}
          {books.length === 0 ? <p className={styles.note}>{t('citations.noBooks')}</p> : null}
        </>
      ) : null}
    </div>
  );
}

function CitationRow({ noteId, onRemove }: { noteId: string; onRemove: () => void }) {
  const { t } = useTranslation(['journal', 'common']);
  const noteQuery = useQuery(bookNoteQuery(noteId));

  return (
    <div className={styles.row}>
      <span className={styles.rowTitle}>{noteQuery.data ? noteBody(noteQuery.data) : t('citations.excerpt')}</span>
      <Button variant="outlined" onClick={onRemove}>
        {t('common:button.cancel')}
      </Button>
    </div>
  );
}
