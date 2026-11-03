import {
  CalendarBlankIcon,
  CaretDoubleLeftIcon,
  CaretDoubleRightIcon,
  CaretLeftIcon,
  LockIcon,
  LockOpenIcon,
  PlusIcon
} from '@phosphor-icons/react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import clsx from 'clsx';
import { useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { JournalEntry, JournalEntryInput, JournalKind } from '@/bridge/journal';
import { journalEntryCreate, journalEntryDelete, journalEntryUpdate } from '@/bridge/journal';
import { placeCreate } from '@/bridge/place';
import { tagCreate } from '@/bridge/tag';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { Checkbox } from '@/components/checkbox';
import { DateField } from '@/components/date-field';
import type { MarkdownEditorHandle } from '@/components/markdown';
import { looseImageOwner, MarkdownEditor, mountLooseImages, retainBodyImages } from '@/components/markdown';
import { useBackLayer } from '@/layout/mobile/back';
import { refreshFiles } from '@/query/files/query';
import { refreshJournal } from '@/query/journal/query';
import { refreshPlaces } from '@/query/place/query';
import { refreshTags } from '@/query/tag/query';
import { fromDateInputValue, startOfLocalDay, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import styles from '../entry.module.css';
import { useJournalNames } from '../names';
import { journalDate } from '../text';
import { Citations } from './citations';
import { MetaChips } from './meta-chips';
import { Promote } from './promote';
import { Seal } from './seal';

type Panel = 'date' | 'seal' | 'place' | 'tag' | 'cite' | 'member';

function AddToggle({ label, open, onClick }: { label: string; open: boolean; onClick: () => void }) {
  const { t } = useTranslation('journal');
  return (
    <IconButton
      label={open ? t('editor.done') : t('editor.addWith', { target: label })}
      size="compact"
      selected={open}
      onClick={onClick}
    >
      <PlusIcon weight="regular" />
    </IconButton>
  );
}

type JournalEditorProps = {
  entry: JournalEntry | null;
  kind: JournalKind;
  onClose: () => void;
  onSaved: (entry: JournalEntry) => void;
  onDeleted?: () => void;
};

/** 手记编辑。类型在新建时定下，正文占满页面，设置从右侧盖住内容区。 */
export function JournalEditor({ entry, kind, onClose, onSaved, onDeleted }: JournalEditorProps) {
  const { t } = useTranslation(['journal', 'common']);
  const queryClient = useQueryClient();
  const names = useJournalNames();
  const markdownRef = useRef<MarkdownEditorHandle>(null);
  const entryKind = entry?.kind ?? kind;
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [panel, setPanel] = useState<Panel | null>(null);
  const [title, setTitle] = useState(entry?.title ?? '');
  const [body, setBody] = useState(entry?.locked ? '' : (entry?.body ?? ''));
  const [occurredAt, setOccurredAt] = useState(toDateInputValue(entry?.occurredAt ?? startOfLocalDay()));
  const [highlight, setHighlight] = useState(entry?.highlight ?? false);
  const [memberIds, setMemberIds] = useState(entry?.memberIds ?? []);
  const [placeIds, setPlaceIds] = useState(entry?.placeIds ?? []);
  const [tagIds, setTagIds] = useState(entry?.tagIds ?? []);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const occurredTime = fromDateInputValue(occurredAt) ?? startOfLocalDay();

  function closeSettings() {
    setSettingsOpen(false);
    setPanel(null);
    setConfirmDelete(false);
  }

  function stepBack() {
    if (panel) {
      setPanel(null);
      return;
    }
    if (settingsOpen) {
      closeSettings();
      return;
    }
    onClose();
  }

  useBackLayer(true, stepBack);

  function toggle(next: Panel) {
    setPanel((current) => (current === next ? null : next));
  }

  function currentBody() {
    return markdownRef.current?.markdown() ?? body;
  }

  function input(): JournalEntryInput {
    const nextBody = currentBody();
    const stillSealed = Boolean(entry?.locked) && nextBody.trim() === '';
    return {
      kind: entryKind,
      occurredAt: occurredTime,
      title,
      body: stillSealed ? (entry?.body ?? '') : nextBody,
      locked: stillSealed,
      highlight,
      memberIds,
      tagIds,
      placeIds
    };
  }

  const save = useMutation({
    mutationFn: async () => {
      const next = input();
      const saved = entry ? await journalEntryUpdate(entry.id, next) : await journalEntryCreate(next);
      if (entry && !entry.locked) {
        await retainBodyImages('journal_entry', entry.id, next.body);
        markdownRef.current?.saved(next.body);
      } else if (!entry) {
        try {
          await mountLooseImages('journal_entry', saved.id, next.body);
        } catch (error) {
          await journalEntryDelete(saved.id).catch(() => undefined);
          throw error;
        }
        markdownRef.current?.saved(next.body);
      }
      return saved;
    },
    onSuccess: async (saved) => {
      await refreshJournal(queryClient);
      await refreshFiles(queryClient);
      onSaved(saved);
    }
  });
  const remove = useMutation({
    mutationFn: () => {
      if (!entry) {
        return Promise.reject(new Error(t('errors.notFound')));
      }
      return journalEntryDelete(entry.id);
    },
    onSuccess: async () => {
      await refreshJournal(queryClient);
      onDeleted?.();
    }
  });
  const busy = save.isPending || remove.isPending;

  return (
    <form
      className={styles.edit}
      onSubmit={(event) => {
        event.preventDefault();
        save.mutate();
      }}
    >
      <header className={styles.bar}>
        <IconButton label={t('common:button.back')} disabled={busy} onClick={stepBack}>
          <CaretLeftIcon weight="regular" />
        </IconButton>
        <Button
          type="submit"
          className={styles.push}
          disabled={busy}
          onClick={() => {
            if (title.trim() === '') {
              closeSettings();
            }
          }}
        >
          {t('common:button.save')}
        </Button>
        <IconButton
          label={settingsOpen ? t('editor.collapseSettings') : t('editor.openSettings')}
          expanded={settingsOpen}
          disabled={busy}
          onClick={() => (settingsOpen ? closeSettings() : setSettingsOpen(true))}
        >
          {settingsOpen ? <CaretDoubleRightIcon weight="regular" /> : <CaretDoubleLeftIcon weight="regular" />}
        </IconButton>
      </header>
      <div className={styles.stage}>
        <div className={styles.sheet}>
          <input
            className={styles.title}
            value={title}
            required
            placeholder={t('editor.title')}
            aria-label={t('editor.title')}
            onChange={(event) => setTitle(event.target.value)}
          />
          {entry?.locked ? <p className={clsx(styles.notice, styles.sheetNotice)}>{t('editor.lockedNotice')}</p> : null}
          {!settingsOpen && save.error ? (
            <p className={clsx(styles.error, styles.sheetNotice)}>{errorMessage(save.error)}</p>
          ) : null}
          <div className={styles.editorWrap}>
            <MarkdownEditor
              ref={markdownRef}
              className={styles.fill}
              height="100%"
              edge
              value={body}
              disabled={Boolean(entry?.locked)}
              placeholder={t('editor.bodyPlaceholder')}
              image={
                entry?.locked ? undefined : entry ? { owner: 'journal_entry', ownerId: entry.id } : looseImageOwner
              }
              onChange={setBody}
            />
          </div>
        </div>
      </div>
      <button
        type="button"
        className={styles.scrim}
        data-open={settingsOpen ? 'true' : undefined}
        tabIndex={-1}
        aria-label={t('editor.collapseSettings')}
        onClick={closeSettings}
      />
      <aside className={styles.drawer} data-open={settingsOpen ? 'true' : undefined} aria-label={t('editor.settings')}>
        <div className={styles.drawerHead}>
          <div className={styles.drawerTitle}>
            <h2 className={styles.drawerHeading}>{t('editor.settings')}</h2>
            <p className={styles.kindName}>{t(`page.${entryKind}`)}</p>
          </div>
        </div>
        <div className={styles.drawerBody}>
          <section className={styles.group}>
            <div className={styles.groupHead}>
              <h2 className={styles.groupTitle}>{t('section.date')}</h2>
              <IconButton
                label={panel === 'date' ? t('editor.done') : t('editor.editDate')}
                size="compact"
                selected={panel === 'date'}
                onClick={() => toggle('date')}
              >
                <CalendarBlankIcon weight="regular" />
              </IconButton>
            </div>
            {panel === 'date' ? (
              <DateField label={t('editor.occurredAt')} value={occurredAt} onChange={setOccurredAt} />
            ) : (
              <p className={styles.value}>{journalDate(occurredTime).full}</p>
            )}
          </section>
          {entryKind === 'diary' ? (
            <section className={styles.group}>
              <div className={styles.groupHead}>
                <h2 className={styles.groupTitle}>{t('section.seal')}</h2>
                {entry ? (
                  <IconButton
                    label={panel === 'seal' ? t('editor.done') : entry.locked ? t('seal.unseal') : t('seal.seal')}
                    size="compact"
                    selected={panel === 'seal'}
                    onClick={() => toggle('seal')}
                  >
                    {entry.locked ? <LockOpenIcon weight="regular" /> : <LockIcon weight="regular" />}
                  </IconButton>
                ) : null}
              </div>
              <p className={styles.value}>
                {entry ? (entry.locked ? t('editor.sealLocked') : t('editor.sealUnlocked')) : t('editor.sealAfterSave')}
              </p>
              {entry && panel === 'seal' ? (
                <Seal entry={entry} readBody={currentBody} onKept={() => markdownRef.current?.saved()} />
              ) : null}
            </section>
          ) : null}
          <section className={styles.group}>
            <div className={styles.groupHead}>
              <h2 className={styles.groupTitle}>{t('section.place')}</h2>
              <AddToggle label={t('section.place')} open={panel === 'place'} onClick={() => toggle('place')} />
            </div>
            <MetaChips
              items={names.places}
              selected={placeIds}
              adding={panel === 'place'}
              empty={t('editor.emptyPlaces')}
              addLabel={t('editor.newPlace')}
              listError={names.placeError ? errorMessage(names.placeError) : ''}
              onChange={setPlaceIds}
              onAdd={async (name) => {
                const place = await placeCreate(name);
                await refreshPlaces(queryClient);
                setPlaceIds((current) => (current.includes(place.id) ? current : [...current, place.id]));
              }}
            />
          </section>
          <section className={styles.group}>
            <div className={styles.groupHead}>
              <h2 className={styles.groupTitle}>{t('section.tag')}</h2>
              <AddToggle label={t('section.tag')} open={panel === 'tag'} onClick={() => toggle('tag')} />
            </div>
            <MetaChips
              items={names.tags}
              selected={tagIds}
              adding={panel === 'tag'}
              empty={t('editor.emptyTags')}
              addLabel={t('editor.newTag')}
              listError={names.tagError ? errorMessage(names.tagError) : ''}
              onChange={setTagIds}
              onAdd={async (name) => {
                const tag = await tagCreate(name);
                await refreshTags(queryClient);
                setTagIds((current) => (current.includes(tag.id) ? current : [...current, tag.id]));
              }}
            />
          </section>
          <section className={styles.group}>
            <div className={styles.groupHead}>
              <h2 className={styles.groupTitle}>{t('section.citation')}</h2>
              {entry ? (
                <AddToggle label={t('section.citation')} open={panel === 'cite'} onClick={() => toggle('cite')} />
              ) : null}
            </div>
            {entry ? (
              <Citations entryId={entry.id} adding={panel === 'cite'} onAdded={() => setPanel(null)} />
            ) : (
              <p className={styles.value}>{t('editor.citationAfterSave')}</p>
            )}
          </section>
          <section className={styles.group}>
            <h2 className={styles.groupTitle}>{t('section.other')}</h2>
            <Checkbox
              label={t('editor.highlight')}
              checked={highlight}
              onChange={(event) => setHighlight(event.target.checked)}
            />
            <div className={styles.groupHead}>
              <h3 className={styles.groupTitle}>{t('section.member')}</h3>
              <AddToggle label={t('section.member')} open={panel === 'member'} onClick={() => toggle('member')} />
            </div>
            <MetaChips
              items={names.members}
              selected={memberIds}
              adding={panel === 'member'}
              empty={t('editor.emptyMembers')}
              listError={names.memberError ? errorMessage(names.memberError) : ''}
              onChange={setMemberIds}
            />
            {entry ? <Promote entry={entry} /> : null}
          </section>
        </div>
        <footer className={styles.drawerFoot}>
          {save.error ? <p className={styles.error}>{errorMessage(save.error)}</p> : null}
          {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
          {confirmDelete ? <p className={styles.hint}>{t('editor.deleteHint')}</p> : null}
          <div className={styles.footActions}>
            {entry && onDeleted ? (
              <Button
                variant="outlined"
                tone="destructive"
                className={styles.footButton}
                disabled={busy}
                onClick={() => {
                  if (!confirmDelete) {
                    setConfirmDelete(true);
                    return;
                  }
                  remove.mutate();
                }}
              >
                {confirmDelete ? t('editor.confirmDelete') : t('common:button.delete')}
              </Button>
            ) : null}
            <Button
              type="submit"
              className={styles.footButton}
              disabled={busy}
              onClick={() => {
                if (title.trim() === '') {
                  closeSettings();
                }
              }}
            >
              {t('common:button.save')}
            </Button>
          </div>
        </footer>
      </aside>
    </form>
  );
}
