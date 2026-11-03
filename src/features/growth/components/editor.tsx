import {
  CalendarBlankIcon,
  CameraIcon,
  CaretDoubleLeftIcon,
  CaretDoubleRightIcon,
  CaretLeftIcon,
  FlagIcon,
  PlusIcon
} from '@phosphor-icons/react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import clsx from 'clsx';
import { useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { GrowthEntry, GrowthEntryInput, GrowthKind } from '@/bridge/growth';
import { growthEntryCreate, growthEntryDelete, growthEntryUpdate } from '@/bridge/growth';
import { placeCreate } from '@/bridge/place';
import { tagCreate } from '@/bridge/tag';
import { Button } from '@/components/button';
import { ChipButton } from '@/components/button/chip';
import { IconButton } from '@/components/button/icon';
import { SegmentButton } from '@/components/button/segment';
import { Checkbox } from '@/components/checkbox';
import { DateField } from '@/components/date-field';
import type { MarkdownEditorHandle } from '@/components/markdown';
import {
  looseImageOwner,
  MarkdownEditor,
  mountLooseImages,
  mountLooseVideos,
  retainBodyImages,
  retainBodyVideos
} from '@/components/markdown';
import { MetaChips } from '@/features/journal/components/meta-chips';
import { useJournalNames } from '@/features/journal/names';
import { useBackLayer } from '@/layout/mobile/back';
import { refreshFiles } from '@/query/files/query';
import { refreshGrowth } from '@/query/growth/query';
import { refreshPlaces } from '@/query/place/query';
import { refreshTags } from '@/query/tag/query';
import { formatDate, fromDateInputValue, startOfLocalDay, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import styles from '../entry.module.css';

const kinds: { kind: GrowthKind; labelKey: 'page.moment' | 'page.milestone'; Icon: typeof FlagIcon }[] = [
  { kind: 'moment', labelKey: 'page.moment', Icon: CameraIcon },
  { kind: 'milestone', labelKey: 'page.milestone', Icon: FlagIcon }
];

type Panel = 'date' | 'member' | 'place' | 'tag';

function AddToggle({ label, open, onClick }: { label: string; open: boolean; onClick: () => void }) {
  const { t } = useTranslation('growth');
  return (
    <IconButton
      label={open ? t('form.done') : t('form.add', { label })}
      size="compact"
      selected={open}
      onClick={onClick}
    >
      <PlusIcon weight="regular" />
    </IconButton>
  );
}

type GrowthEditorProps = {
  entry: GrowthEntry | null;
  ownerId: string;
  kind: GrowthKind;
  onClose: () => void;
  onSaved: (entry: GrowthEntry) => void;
  onDeleted?: () => void;
};

/** 成长编辑。正文铺满，属于谁、一起的人和封存都在右侧抽屉。 */
export function GrowthEditor({ entry, ownerId, kind, onClose, onSaved, onDeleted }: GrowthEditorProps) {
  const { t } = useTranslation(['growth', 'common']);
  const queryClient = useQueryClient();
  const names = useJournalNames();
  const markdownRef = useRef<MarkdownEditorHandle>(null);
  const keptBody = useRef(entry?.body ?? '');
  const lockedRef = useRef(entry?.locked ?? false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [panel, setPanel] = useState<Panel | null>(null);
  const [entryKind, setEntryKind] = useState<GrowthKind>(entry?.kind ?? kind);
  const [title, setTitle] = useState(entry?.title ?? '');
  const [body, setBody] = useState(entry?.locked ? '' : (entry?.body ?? ''));
  const [locked, setLocked] = useState(entry?.locked ?? false);
  const [occurredAt, setOccurredAt] = useState(toDateInputValue(entry?.occurredAt ?? startOfLocalDay()));
  const [highlight, setHighlight] = useState(entry?.highlight ?? false);
  const [memberId, setMemberId] = useState(entry?.memberId ?? ownerId);
  const [companionIds, setCompanionIds] = useState(
    (entry?.memberIds ?? []).filter((id) => id !== (entry?.memberId ?? ownerId))
  );
  const [placeIds, setPlaceIds] = useState(entry?.placeIds ?? []);
  const [tagIds, setTagIds] = useState(entry?.tagIds ?? []);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const occurredTime = fromDateInputValue(occurredAt) ?? startOfLocalDay();
  const companions = names.members.filter((item) => item.id !== memberId);

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

  function onBody(next: string) {
    setBody(next);
    if (!lockedRef.current) {
      keptBody.current = next;
    }
  }

  function changeLocked(next: boolean) {
    if (next) {
      keptBody.current = markdownRef.current?.markdown() ?? keptBody.current;
    }
    lockedRef.current = next;
    setLocked(next);
    setBody(next ? '' : keptBody.current);
  }

  function currentBody() {
    if (lockedRef.current) {
      return keptBody.current;
    }
    return markdownRef.current?.markdown() ?? body;
  }

  function input(): GrowthEntryInput {
    return {
      kind: entryKind,
      memberId,
      occurredAt: occurredTime,
      title,
      body: currentBody(),
      locked: lockedRef.current,
      highlight,
      memberIds: companionIds.filter((id) => id !== memberId),
      tagIds,
      placeIds
    };
  }

  const save = useMutation({
    mutationFn: async () => {
      const next = input();
      const saved = entry ? await growthEntryUpdate(entry.id, next) : await growthEntryCreate(next);
      if (entry) {
        await retainBodyImages('growth_entry', entry.id, next.body);
        await retainBodyVideos('growth_entry', entry.id, next.body);
        markdownRef.current?.saved(next.body);
      } else {
        try {
          await mountLooseImages('growth_entry', saved.id, next.body);
          await mountLooseVideos('growth_entry', saved.id, next.body);
        } catch (error) {
          await growthEntryDelete(saved.id).catch(() => undefined);
          throw error;
        }
        markdownRef.current?.saved(next.body);
      }
      return saved;
    },
    onSuccess: async (saved) => {
      await refreshGrowth(queryClient);
      await refreshFiles(queryClient);
      onSaved(saved);
    }
  });
  const remove = useMutation({
    mutationFn: () => {
      if (!entry) {
        return Promise.reject(new Error(t('page.notFound')));
      }
      return growthEntryDelete(entry.id);
    },
    onSuccess: async () => {
      await refreshGrowth(queryClient);
      onDeleted?.();
    }
  });
  const busy = save.isPending || remove.isPending;
  const mediaOwner = locked
    ? undefined
    : entry
      ? { owner: 'growth_entry' as const, ownerId: entry.id }
      : looseImageOwner;

  return (
    <form
      className={styles.edit}
      onSubmit={(event) => {
        event.preventDefault();
        if (memberId === '') {
          return;
        }
        save.mutate();
      }}
    >
      <header className={styles.bar}>
        <IconButton label={t('page.back')} disabled={busy} onClick={stepBack}>
          <CaretLeftIcon weight="regular" />
        </IconButton>
        <Button
          type="submit"
          className={styles.push}
          disabled={busy || memberId === ''}
          onClick={() => {
            if (title.trim() === '') {
              closeSettings();
            }
          }}
        >
          {t('common:button.save')}
        </Button>
        <IconButton
          label={settingsOpen ? t('page.closeSettings') : t('page.openSettings')}
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
            placeholder={t('form.title')}
            aria-label={t('form.title')}
            onChange={(event) => setTitle(event.target.value)}
          />
          {locked ? <p className={styles.note}>{t('page.sealed')}</p> : null}
          {!settingsOpen && save.error ? (
            <p className={clsx(styles.error, styles.sheetNotice)}>{errorMessage(save.error)}</p>
          ) : null}
          <div className={styles.editorWrap}>
            <MarkdownEditor
              key={locked ? 'sealed' : 'open'}
              ref={markdownRef}
              className={styles.fill}
              height="100%"
              edge
              value={body}
              disabled={locked}
              placeholder={t('form.bodyPlaceholder')}
              image={mediaOwner}
              video={mediaOwner}
              onChange={onBody}
            />
          </div>
        </div>
      </div>
      <button
        type="button"
        className={styles.scrim}
        data-open={settingsOpen ? 'true' : undefined}
        tabIndex={-1}
        aria-label={t('page.closeSettings')}
        onClick={closeSettings}
      />
      <aside className={styles.drawer} data-open={settingsOpen ? 'true' : undefined} aria-label={t('page.settings')}>
        <div className={styles.drawerHead}>
          <div className={styles.drawerTitle}>
            <h2 className={styles.drawerHeading}>{t('page.settings')}</h2>
          </div>
        </div>
        <div className={styles.drawerBody}>
          <section className={styles.group}>
            <h2 className={styles.groupTitle}>{t('form.kind')}</h2>
            <div className={styles.segments} role="tablist" aria-label={t('page.typeAria')}>
              {kinds.map((item) => (
                <SegmentButton
                  key={item.kind}
                  selected={entryKind === item.kind}
                  onClick={() => setEntryKind(item.kind)}
                >
                  <item.Icon weight="regular" />
                  {t(item.labelKey)}
                </SegmentButton>
              ))}
            </div>
          </section>
          <section className={styles.group}>
            <div className={styles.groupHead}>
              <h2 className={styles.groupTitle}>{t('form.date')}</h2>
              <IconButton
                label={panel === 'date' ? t('form.done') : t('form.editDate')}
                size="compact"
                selected={panel === 'date'}
                onClick={() => toggle('date')}
              >
                <CalendarBlankIcon weight="regular" />
              </IconButton>
            </div>
            {panel === 'date' ? (
              <DateField label={t('form.occurredAt')} value={occurredAt} onChange={setOccurredAt} />
            ) : (
              <p className={styles.value}>{formatDate(occurredTime)}</p>
            )}
          </section>
          <section className={styles.group}>
            <h2 className={styles.groupTitle}>{t('form.owner')}</h2>
            {names.memberError ? <p className={styles.error}>{errorMessage(names.memberError)}</p> : null}
            {names.members.length === 0 ? <p className={styles.value}>{t('form.noMembers')}</p> : null}
            <div className={styles.chips}>
              {names.members.map((item) => (
                <ChipButton
                  key={item.id}
                  tone="growth"
                  selected={memberId === item.id}
                  onClick={() => {
                    setMemberId(item.id);
                    setCompanionIds((current) => current.filter((id) => id !== item.id));
                  }}
                >
                  {item.name}
                </ChipButton>
              ))}
            </div>
          </section>
          <section className={styles.group}>
            <div className={styles.groupHead}>
              <h2 className={styles.groupTitle}>{t('form.companions')}</h2>
              <AddToggle label={t('form.companions')} open={panel === 'member'} onClick={() => toggle('member')} />
            </div>
            <MetaChips
              items={companions}
              selected={companionIds}
              adding={panel === 'member'}
              empty={t('form.noCompanions')}
              onChange={setCompanionIds}
            />
          </section>
          <section className={styles.group}>
            <div className={styles.groupHead}>
              <h2 className={styles.groupTitle}>{t('form.places')}</h2>
              <AddToggle label={t('form.places')} open={panel === 'place'} onClick={() => toggle('place')} />
            </div>
            <MetaChips
              items={names.places}
              selected={placeIds}
              adding={panel === 'place'}
              empty={t('form.noPlaces')}
              addLabel={t('form.newPlace')}
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
              <h2 className={styles.groupTitle}>{t('form.tags')}</h2>
              <AddToggle label={t('form.tags')} open={panel === 'tag'} onClick={() => toggle('tag')} />
            </div>
            <MetaChips
              items={names.tags}
              selected={tagIds}
              adding={panel === 'tag'}
              empty={t('form.noTags')}
              addLabel={t('form.newTag')}
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
            <h2 className={styles.groupTitle}>{t('form.other')}</h2>
            <Checkbox
              label={t('form.highlight')}
              checked={highlight}
              onChange={(event) => setHighlight(event.target.checked)}
            />
            {entry ? (
              <Checkbox
                label={t('form.seal')}
                checked={locked}
                onChange={(event) => changeLocked(event.target.checked)}
              />
            ) : (
              <p className={styles.value}>{t('form.sealLater')}</p>
            )}
          </section>
        </div>
        <footer className={styles.drawerFoot}>
          {save.error ? <p className={styles.error}>{errorMessage(save.error)}</p> : null}
          {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
          {confirmDelete ? <p className={styles.hint}>{t('form.confirmDeleteEntry')}</p> : null}
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
                {confirmDelete ? t('form.confirmDelete') : t('common:button.delete')}
              </Button>
            ) : null}
            <Button
              type="submit"
              className={styles.footButton}
              disabled={busy || memberId === ''}
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
