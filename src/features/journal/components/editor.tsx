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

const kindLabel: Record<JournalKind, string> = {
  diary: '日记',
  spark: '灵感',
  writing: '写作'
};

type Panel = 'date' | 'seal' | 'place' | 'tag' | 'cite' | 'member';

function AddToggle({ label, open, onClick }: { label: string; open: boolean; onClick: () => void }) {
  return (
    <IconButton label={open ? '完成' : `添加${label}`} size="compact" selected={open} onClick={onClick}>
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
        return Promise.reject(new Error('手记不存在'));
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
        <IconButton label="返回" disabled={busy} onClick={onClose}>
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
          保存
        </Button>
        <IconButton
          label={settingsOpen ? '收起设置' : '打开设置'}
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
            placeholder="标题"
            aria-label="标题"
            onChange={(event) => setTitle(event.target.value)}
          />
          {entry?.locked ? (
            <p className={clsx(styles.notice, styles.sheetNotice)}>正文已封存。解开之后才能改。</p>
          ) : null}
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
              placeholder="写下来"
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
        aria-label="收起设置"
        onClick={closeSettings}
      />
      <aside className={styles.drawer} data-open={settingsOpen ? 'true' : undefined} aria-label="设置">
        <div className={styles.drawerHead}>
          <div className={styles.drawerTitle}>
            <h2 className={styles.drawerHeading}>设置</h2>
            <p className={styles.kindName}>{kindLabel[entryKind]}</p>
          </div>
        </div>
        <div className={styles.drawerBody}>
          <section className={styles.group}>
            <div className={styles.groupHead}>
              <h2 className={styles.groupTitle}>日期</h2>
              <IconButton
                label={panel === 'date' ? '完成' : '修改日期'}
                size="compact"
                selected={panel === 'date'}
                onClick={() => toggle('date')}
              >
                <CalendarBlankIcon weight="regular" />
              </IconButton>
            </div>
            {panel === 'date' ? (
              <DateField label="发生时间" value={occurredAt} onChange={setOccurredAt} />
            ) : (
              <p className={styles.value}>{journalDate(occurredTime).full}</p>
            )}
          </section>
          {entryKind === 'diary' ? (
            <section className={styles.group}>
              <div className={styles.groupHead}>
                <h2 className={styles.groupTitle}>封存</h2>
                {entry ? (
                  <IconButton
                    label={panel === 'seal' ? '完成' : entry.locked ? '解开' : '封存'}
                    size="compact"
                    selected={panel === 'seal'}
                    onClick={() => toggle('seal')}
                  >
                    {entry.locked ? <LockOpenIcon weight="regular" /> : <LockIcon weight="regular" />}
                  </IconButton>
                ) : null}
              </div>
              <p className={styles.value}>
                {entry ? (entry.locked ? '正文已封存。' : '正文还没有封存。') : '保存之后可以封存。'}
              </p>
              {entry && panel === 'seal' ? (
                <Seal entry={entry} readBody={currentBody} onKept={() => markdownRef.current?.saved()} />
              ) : null}
            </section>
          ) : null}
          <section className={styles.group}>
            <div className={styles.groupHead}>
              <h2 className={styles.groupTitle}>地点</h2>
              <AddToggle label="地点" open={panel === 'place'} onClick={() => toggle('place')} />
            </div>
            <MetaChips
              items={names.places}
              selected={placeIds}
              adding={panel === 'place'}
              empty="还没有地点。"
              addLabel="新地点"
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
              <h2 className={styles.groupTitle}>标签</h2>
              <AddToggle label="标签" open={panel === 'tag'} onClick={() => toggle('tag')} />
            </div>
            <MetaChips
              items={names.tags}
              selected={tagIds}
              adding={panel === 'tag'}
              empty="还没有标签。"
              addLabel="新标签"
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
              <h2 className={styles.groupTitle}>引用</h2>
              {entry ? <AddToggle label="引用" open={panel === 'cite'} onClick={() => toggle('cite')} /> : null}
            </div>
            {entry ? (
              <Citations entryId={entry.id} adding={panel === 'cite'} onAdded={() => setPanel(null)} />
            ) : (
              <p className={styles.value}>保存之后可以引用。</p>
            )}
          </section>
          <section className={styles.group}>
            <h2 className={styles.groupTitle}>其他</h2>
            <Checkbox label="标为高光" checked={highlight} onChange={(event) => setHighlight(event.target.checked)} />
            <div className={styles.groupHead}>
              <h3 className={styles.groupTitle}>成员</h3>
              <AddToggle label="成员" open={panel === 'member'} onClick={() => toggle('member')} />
            </div>
            <MetaChips
              items={names.members}
              selected={memberIds}
              adding={panel === 'member'}
              empty="还没有成员。"
              listError={names.memberError ? errorMessage(names.memberError) : ''}
              onChange={setMemberIds}
            />
            {entry ? <Promote entry={entry} /> : null}
          </section>
        </div>
        <footer className={styles.drawerFoot}>
          {save.error ? <p className={styles.error}>{errorMessage(save.error)}</p> : null}
          {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
          {confirmDelete ? <p className={styles.hint}>再点一次就会删掉这篇。</p> : null}
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
                {confirmDelete ? '确认删除' : '删除'}
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
              保存
            </Button>
          </div>
        </footer>
      </aside>
    </form>
  );
}
