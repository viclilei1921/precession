import { LightningIcon, MagnifyingGlassIcon, NotebookIcon, PencilLineIcon, PlusIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import type { JournalKind } from '@/bridge/journal';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { SegmentButton } from '@/components/button/segment';
import { firstImageId, UploadThumb } from '@/components/markdown';
import { Scroll } from '@/components/scroll';
import { usePageSearch } from '@/hooks/search';
import { Fab } from '@/layout/mobile/fab';
import { journalListQuery } from '@/query/journal/query';
import { PATH } from '@/router/path';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import { matchesQuery } from '@/utils/search';
import { JournalEditor } from './components/editor';
import { MobileJournalEditor } from './mobile/editor';
import { useJournalNames } from './names';
import styles from './page.module.css';
import { journalDate, journalExcerpt, journalWords, lookupNames } from './text';

const kinds: { kind: JournalKind; label: string; Icon: typeof NotebookIcon }[] = [
  { kind: 'diary', label: '日记', Icon: NotebookIcon },
  { kind: 'spark', label: '灵感', Icon: LightningIcon },
  { kind: 'writing', label: '写作', Icon: PencilLineIcon }
];

/** 手记列表。按类型切换，每条是一张卡片。 */
export function JournalPage() {
  const navigate = useNavigate();
  const search = useSearch({ from: '/journal' });
  const mobile = useAppStore((state) => state.isMobile());
  const [kind, setKind] = useState<JournalKind>(search.kind ?? 'diary');
  const [creating, setCreating] = useState(false);
  const [finding, setFinding] = useState(false);
  const [localQuery, setLocalQuery] = useState('');
  const listQuery = useQuery(journalListQuery(kind));
  const names = useJournalNames();
  const pageQuery = usePageSearch('搜索手记');
  const query = mobile ? localQuery : pageQuery;
  const entries = [...(listQuery.data ?? [])]
    .filter((entry) => matchesQuery(query, entry.title, entry.body))
    .sort((left, right) => right.occurredAt - left.occurredAt);

  useEffect(() => {
    if (search.kind) {
      setKind(search.kind);
    }
    if (mobile) {
      if (search.kind) {
        void navigate({
          to: PATH.journal,
          search: search.create ? { create: true } : {},
          replace: true
        });
      }
      return;
    }
    if (search.create) {
      setCreating(true);
    }
    if (search.create || search.kind) {
      void navigate({ to: PATH.journal, search: {}, replace: true });
    }
  }, [mobile, navigate, search.create, search.kind]);

  if (mobile && search.create) {
    return (
      <MobileJournalEditor
        kind={kind}
        onClose={() => void navigate({ to: PATH.journal, search: {} })}
        onSaved={(entry) => {
          void navigate({ to: PATH.journalEntry, params: { entryId: entry.id } });
        }}
      />
    );
  }

  if (creating) {
    return (
      <JournalEditor
        entry={null}
        kind={kind}
        onClose={() => setCreating(false)}
        onSaved={(entry) => {
          setCreating(false);
          void navigate({ to: PATH.journalEntry, params: { entryId: entry.id } });
        }}
      />
    );
  }

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <div className={styles.segments} role="tablist" aria-label="手记类型">
          {kinds.map((item) => (
            <SegmentButton key={item.kind} selected={kind === item.kind} onClick={() => setKind(item.kind)}>
              <item.Icon weight="regular" />
              {item.label}
            </SegmentButton>
          ))}
        </div>
        {mobile ? (
          <IconButton
            className={styles.add}
            label="搜索手记"
            onClick={() =>
              setFinding((current) => {
                if (current) {
                  setLocalQuery('');
                }
                return !current;
              })
            }
          >
            <MagnifyingGlassIcon weight="regular" />
          </IconButton>
        ) : (
          <Button className={styles.add} onClick={() => setCreating(true)}>
            <PlusIcon weight="regular" />
            新建
          </Button>
        )}
      </header>
      {mobile && finding ? (
        <form
          className={styles.search}
          onSubmit={(event) => {
            event.preventDefault();
          }}
        >
          <input
            type="search"
            value={localQuery}
            placeholder="搜索标题或正文"
            aria-label="搜索手记"
            onChange={(event) => setLocalQuery(event.target.value)}
          />
          <button
            type="button"
            onClick={() => {
              setLocalQuery('');
              setFinding(false);
            }}
          >
            取消
          </button>
        </form>
      ) : null}
      <Scroll className={styles.scroll}>
        <div className={styles.body}>
          {listQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}
          {listQuery.error ? <p className={styles.error}>{errorMessage(listQuery.error)}</p> : null}
          {!listQuery.isPending && !listQuery.error && entries.length === 0 ? (
            <p className={styles.empty}>{query.trim() ? '没有匹配的手记。' : '这里还没有记录。'}</p>
          ) : null}
          {entries.length > 0 ? (
            <ul className={styles.list}>
              {entries.map((entry) => {
                const date = journalDate(entry.occurredAt);
                const meta = [
                  ...lookupNames(entry.placeIds, names.places),
                  ...lookupNames(entry.memberIds, names.members),
                  ...lookupNames(entry.tagIds, names.tags)
                ].join(' · ');
                const words = journalWords(entry.body, entry.locked);
                const photo = entry.locked ? '' : firstImageId(entry.body);
                return (
                  <li key={entry.id}>
                    <button
                      type="button"
                      className={styles.card}
                      data-photo={photo ? 'true' : undefined}
                      onClick={() => void navigate({ to: PATH.journalEntry, params: { entryId: entry.id } })}
                    >
                      <span className={styles.date}>
                        <span className={styles.day}>{date.day}</span>
                        <span className={styles.month}>{date.month}</span>
                        <span className={styles.week}>{date.week}</span>
                      </span>
                      <span className={styles.cardMain}>
                        <span className={styles.cardTop}>
                          <span className={styles.cardTitle}>{entry.title}</span>
                          {entry.highlight || entry.locked ? (
                            <span className={styles.badges}>
                              {entry.highlight ? (
                                <span className={styles.badge} data-tone="highlight">
                                  高光
                                </span>
                              ) : null}
                              {entry.locked ? <span className={styles.badge}>已封存</span> : null}
                            </span>
                          ) : null}
                        </span>
                        <span className={styles.excerpt}>{journalExcerpt(entry.body, entry.locked)}</span>
                        <span className={styles.cardFoot}>
                          {meta ? <span className={styles.quiet}>{meta}</span> : null}
                          {words != null ? <span className={styles.words}>{words} 字</span> : null}
                        </span>
                      </span>
                      {photo ? (
                        <UploadThumb id={photo} owner="journal_entry" ownerId={entry.id} className={styles.thumb} />
                      ) : null}
                    </button>
                  </li>
                );
              })}
            </ul>
          ) : null}
        </div>
      </Scroll>
      {mobile ? (
        <Fab label="写一条" onClick={() => void navigate({ to: PATH.journal, search: { create: true } })} />
      ) : null}
    </section>
  );
}
