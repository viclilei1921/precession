import { ExportIcon, PlusIcon, TrashIcon } from '@phosphor-icons/react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useState } from 'react';
import { pickDirectory, pickFiles } from '@/bridge/files';
import type { UploadCatalog } from '@/bridge/upload';
import { uploadDelete, uploadExport } from '@/bridge/upload';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { SegmentButton } from '@/components/button/segment';
import { Modal } from '@/components/modal';
import { PasswordInput } from '@/components/password-input';
import { Scroll } from '@/components/scroll';
import { usePageSearch } from '@/hooks/search';
import { filesCatalogQuery, refreshFileOwner, refreshFiles } from '@/query/files/query';
import { PATH } from '@/router/path';
import { formatMonthDay } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import type { ImportDraft } from './import-dialog';
import { ImportDialog } from './import-dialog';
import styles from './page.module.css';
import { Player } from './player';
import type { FileModule, FileSort, KindFilter } from './text';
import {
  canOpen,
  compareFiles,
  countForKind,
  deleteCopy,
  displayName,
  FILE_MODULES,
  fileBaseName,
  fileRole,
  formatBytes,
  isSealedFileName,
  KIND_FILTERS,
  kindFromName,
  kindLabel,
  matchesFile,
  moduleLabel,
  moduleOf,
  openLabel,
  plainFileName,
  readingNote,
  recordTitle
} from './text';
import { FileThumb } from './thumb';
import { MediaViewer } from './viewer';

export function FilesPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const query = usePageSearch('搜索文件名或记录');
  const catalog = useQuery(filesCatalogQuery);
  const files = catalog.data ?? [];
  const [module, setModule] = useState<FileModule | 'all'>('all');
  const [kind, setKind] = useState<KindFilter>('all');
  const [sort, setSort] = useState<FileSort>('recent');
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [previewId, setPreviewId] = useState<string | null>(null);
  const [playOnOpen, setPlayOnOpen] = useState(false);
  const [previewSync, setPreviewSync] = useState(0);
  const [pending, setPending] = useState<UploadCatalog | null>(null);
  const [imports, setImports] = useState<ImportDraft[] | null>(null);
  const filters = { module, kind, state: 'all' as const, query };
  const visible = files
    .filter((file) => matchesFile(file, files, filters))
    .sort((left, right) => compareFiles(left, right, sort, files));
  const selected = files.find((file) => file.id === selectedId) ?? null;
  const remove = useMutation({
    mutationFn: (file: UploadCatalog) => uploadDelete(file.id),
    onSuccess: async (_result, file) => {
      queryClient.removeQueries({ queryKey: ['files', 'image', file.id] });
      await refreshFiles(queryClient);
      await refreshFileOwner(queryClient, file.owner);
      setPending(null);
      setSelectedId((current) => (current === file.id ? null : current));
      setPreviewId((current) => (current === file.id ? null : current));
    }
  });

  const usage = FILE_MODULES.map((item) => {
    const owned = files.filter((file) => moduleOf(file.owner) === item.id);
    return {
      ...item,
      size: owned.reduce((sum, file) => sum + file.size, 0),
      count: owned.length
    };
  });
  const usageTotal = usage.reduce((sum, item) => sum + item.size, 0);

  function openRecord(file: UploadCatalog) {
    if (!canOpen(file)) {
      return;
    }
    if (file.owner === 'journal_entry') {
      void navigate({ to: PATH.journalEntry, params: { entryId: file.ownerId } });
      return;
    }
    if (file.owner === 'book') {
      void navigate({ to: PATH.bookReader, params: { bookId: file.ownerId }, search: {} });
      return;
    }
    if (file.owner === 'book_note') {
      void navigate({ to: PATH.bookReader, params: { bookId: file.parentId }, search: { note: file.ownerId } });
      return;
    }
    if (file.owner === 'plan') {
      void navigate({ to: PATH.plan, search: { id: file.ownerId } });
      return;
    }
    if (file.owner === 'growth_entry') {
      void navigate({ to: PATH.growthEntry, params: { entryId: file.ownerId } });
      return;
    }
    if (file.owner === 'member') {
      void navigate({ to: PATH.growth, search: { member: file.ownerId } });
    }
  }

  async function beginImport() {
    const paths = await pickFiles({});
    if (paths.length === 0) {
      return;
    }
    setImports(
      paths.map((path) => {
        const raw = fileBaseName(path);
        const sealed = isSealedFileName(raw);
        return { path, name: sealed ? plainFileName(raw) : raw, kind: kindFromName(path), sealed };
      })
    );
  }

  const previewFile = files.find((item) => item.id === previewId) ?? null;
  const previewMedia =
    previewFile && (previewFile.kind === 'image' || previewFile.kind === 'video') ? previewFile : null;

  return (
    <section className={styles.page}>
      <section className={styles.usage} aria-label="占用">
        <div className={styles.usageTop}>
          <strong>占用</strong>
          <span>
            {files.length} 个 · {formatBytes(usageTotal)}
          </span>
          <Button className={styles.upload} onClick={() => void beginImport()}>
            <PlusIcon weight="regular" />
            上传
          </Button>
        </div>
        <div className={styles.bar}>
          {usage
            .filter((item) => item.size > 0)
            .map((item) => (
              <i key={item.id} data-module={item.id} style={{ flex: item.size }} />
            ))}
        </div>
        <div className={styles.legend}>
          {usage.map((item) => {
            const on = module === item.id;
            return (
              <button
                key={item.id}
                type="button"
                data-module={item.id}
                aria-pressed={on}
                data-dim={module !== 'all' && !on ? 'true' : undefined}
                onClick={() => setModule(on ? 'all' : item.id)}
              >
                <i className={styles.dot} data-module={item.id} />
                {item.label} {formatBytes(item.size)}
              </button>
            );
          })}
        </div>
      </section>

      <div className={styles.work}>
        <article className={styles.files} aria-label="文件列表">
          <div className={styles.tools}>
            <div className={styles.segments} role="tablist" aria-label="文件类别">
              {KIND_FILTERS.map((item) => (
                <SegmentButton
                  key={item.id}
                  className={styles.kind}
                  selected={kind === item.id}
                  onClick={() => setKind(item.id)}
                >
                  {item.label}
                  <em>{countForKind(files, filters, item.id)}</em>
                </SegmentButton>
              ))}
            </div>
            <label className={styles.sort}>
              排序
              <select aria-label="排序" value={sort} onChange={(event) => setSort(event.target.value as FileSort)}>
                <option value="recent">最近导入</option>
                <option value="size">体积</option>
                <option value="name">名称</option>
              </select>
            </label>
          </div>
          <Scroll className={styles.listScroll} axis="vertical" gutter>
            {catalog.error ? <p className={styles.error}>{errorMessage(catalog.error)}</p> : null}
            {catalog.isPending ? <p className={styles.empty}>正在读取…</p> : null}
            {!catalog.isPending && !catalog.error && visible.length === 0 ? (
              <p className={styles.empty}>
                {files.length === 0 ? '还没有文件。可以从占用里上传。' : '没有符合的文件。'}
              </p>
            ) : null}
            {visible.map((file) => (
              <FileRow
                key={file.id}
                file={file}
                files={files}
                selected={file.id === selectedId}
                onSelect={() => {
                  setPreviewId(null);
                  setSelectedId(file.id);
                }}
              />
            ))}
          </Scroll>
        </article>
        <div className={styles.pane}>
          {selected ? (
            <>
              <div className={styles.fit}>
                <Player
                  key={selected.id}
                  file={selected}
                  sync={previewSync}
                  onPreview={() => {
                    setPreviewId(selected.id);
                    setPlayOnOpen(selected.kind === 'video');
                  }}
                />
              </div>
              <Scroll className={styles.meta} axis="vertical" gutter>
                <div className={styles.metaBody}>
                  <Inspector
                    key={selected.id}
                    file={selected}
                    files={files}
                    onOpen={() => openRecord(selected)}
                    onDelete={() => {
                      remove.reset();
                      setPending(selected);
                    }}
                  />
                </div>
              </Scroll>
            </>
          ) : (
            <div className={styles.idle}>选一个文件</div>
          )}
        </div>
      </div>

      {previewMedia ? (
        <MediaViewer
          file={previewMedia}
          catalog={files}
          files={visible}
          autoPlay={playOnOpen && previewMedia.kind === 'video'}
          onSelect={(id) => {
            setSelectedId(id);
            setPreviewId(id);
            setPlayOnOpen(false);
          }}
          onClose={() => {
            setPreviewId(null);
            setPreviewSync((current) => current + 1);
          }}
        />
      ) : null}
      {imports ? (
        <ImportDialog
          files={imports}
          onClose={() => setImports(null)}
          onImported={(id) => {
            setImports(null);
            if (id) {
              setSelectedId(id);
            }
          }}
        />
      ) : null}
      {pending ? (
        <Modal title={deleteCopy(pending, files).title} onClose={() => setPending(null)}>
          <p className={styles.dialogBody}>{deleteCopy(pending, files).body}</p>
          {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
          <div className={styles.dialogActions}>
            <Button variant="text" onClick={() => setPending(null)}>
              取消
            </Button>
            <Button tone="destructive" disabled={remove.isPending} onClick={() => remove.mutate(pending)}>
              {remove.isPending ? '正在删除…' : '删除'}
            </Button>
          </div>
        </Modal>
      ) : null}
    </section>
  );
}

function FileRow({
  file,
  files,
  selected,
  onSelect
}: {
  file: UploadCatalog;
  files: UploadCatalog[];
  selected: boolean;
  onSelect: () => void;
}) {
  const role = file.kind === 'book' ? fileRole(file, files) : file.mime === 'image/avif' ? 'AVIF' : file.mime;
  const bits = [moduleLabel(moduleOf(file.owner)), recordTitle(file), role, formatMonthDay(file.createdAt)].filter(
    Boolean
  );
  return (
    <button type="button" className={styles.file} data-selected={selected ? 'true' : undefined} onClick={onSelect}>
      <FileThumb file={file} />
      <span className={styles.fileCopy}>
        <span className={styles.fileName}>{displayName(file, files)}</span>
        <span className={styles.fileMeta}>
          <i className={styles.dot} data-module={moduleOf(file.owner)} />
          <span>{bits.join(' · ')}</span>
        </span>
      </span>
      <span className={styles.fileSize}>{formatBytes(file.size)}</span>
    </button>
  );
}

function Inspector({
  file,
  files,
  onOpen,
  onDelete
}: {
  file: UploadCatalog;
  files: UploadCatalog[];
  onOpen: () => void;
  onDelete: () => void;
}) {
  const plan = deleteCopy(file, files);
  const role = fileRole(file, files);
  const reading = readingNote(file, files);
  const label = kindLabel(file);
  const loose = file.owner === 'loose';
  const opened = canOpen(file);
  const [exporting, setExporting] = useState(false);
  const [exportError, setExportError] = useState<string | null>(null);
  const [exported, setExported] = useState<string | null>(null);
  const [ask, setAsk] = useState<string | null>(null);
  const [password, setPassword] = useState('');

  async function exportTo(dir: string, secret: string) {
    setExporting(true);
    setExportError(null);
    setExported(null);
    try {
      const saved = await uploadExport(file.id, dir, secret);
      setExported(saved);
      setAsk(null);
      setPassword('');
    } catch (error) {
      const message = errorMessage(error);
      if (message === '请输入密码') {
        setAsk(dir);
        return;
      }
      setExportError(message);
    } finally {
      setExporting(false);
    }
  }

  async function beginExport() {
    const dir = await pickDirectory();
    if (!dir) {
      return;
    }
    await exportTo(dir, '');
  }

  return (
    <>
      <h2 className={styles.paneTitle}>{displayName(file, files)}</h2>
      <div className={styles.chips}>
        <span className={styles.chip} data-module={moduleOf(file.owner)}>
          {moduleLabel(moduleOf(file.owner))}
        </span>
        {role ? (
          <span className={styles.chip} data-tone={role === '当前正文' ? 'muted' : 'warn'}>
            {role}
          </span>
        ) : null}
        {file.ownerSealed ? (
          <span className={styles.chip} data-tone="muted">
            已封存
          </span>
        ) : null}
      </div>
      {file.ownerSealed ? <p className={styles.quiet}>这篇记录已封存。这里不显示内容，也不能删除。</p> : null}
      {!loose && !opened ? <p className={styles.quiet}>所属记录已经不在。可以删掉这份残留。</p> : null}
      <dl className={styles.facts}>
        <div>
          <dt>大小</dt>
          <dd>{formatBytes(file.size)}</dd>
        </div>
        <div>
          <dt>类型</dt>
          <dd>{file.mime || '未知'}</dd>
        </div>
        <div>
          <dt>导入</dt>
          <dd>{file.createdAt ? formatMonthDay(file.createdAt) : '未知'}</dd>
        </div>
        <div>
          <dt>路径</dt>
          <dd>
            <code>{file.relPath}</code>
          </dd>
        </div>
        {file.kind === 'image' ? (
          <div>
            <dt>文件名</dt>
            <dd>
              <code>{file.name}</code>
            </dd>
          </div>
        ) : null}
      </dl>
      {!loose ? (
        <div className={styles.ownerCard}>
          <div className={styles.chips}>
            <span className={styles.chip} data-module={moduleOf(file.owner)}>
              {moduleLabel(moduleOf(file.owner))}
              {label ? ` · ${label}` : ''}
            </span>
          </div>
          <strong>{recordTitle(file)}</strong>
          <p className={styles.quiet}>
            {file.owner === 'book'
              ? reading
              : [file.ownerSealed ? '已封存' : '', file.ownerAt ? formatMonthDay(file.ownerAt) : '', file.parentTitle]
                  .filter(Boolean)
                  .join(' · ')}
          </p>
        </div>
      ) : null}
      <div className={styles.paneActions}>
        {!loose ? (
          <Button disabled={!opened} onClick={onOpen}>
            {opened ? openLabel(file) : '记录已不在'}
          </Button>
        ) : null}
        <Button variant="text" disabled={file.ownerSealed || exporting} onClick={() => void beginExport()}>
          <ExportIcon weight="regular" />
          {exporting ? '正在导出…' : '导出'}
        </Button>
        <IconButton label="删除文件" tone="destructive" disabled={plan.blocked} onClick={onDelete}>
          <TrashIcon weight="regular" />
        </IconButton>
      </div>
      {exported ? <p className={styles.quiet}>已导出到 {exported}</p> : null}
      {exportError && !ask ? <p className={styles.error}>{exportError}</p> : null}
      {plan.blocked ? <p className={styles.quiet}>{plan.body}</p> : null}
      {ask ? (
        <Modal title="导出这份文件" onClose={() => setAsk(null)}>
          <p className={styles.dialogBody}>这份文件已加密。输入密码后写到所选文件夹。</p>
          <PasswordInput label="密码" value={password} onChange={(event) => setPassword(event.target.value)} />
          {exportError ? <p className={styles.error}>{exportError}</p> : null}
          <div className={styles.dialogActions}>
            <Button variant="text" onClick={() => setAsk(null)}>
              取消
            </Button>
            <Button disabled={exporting || password.length === 0} onClick={() => void exportTo(ask, password)}>
              {exporting ? '正在导出…' : '导出'}
            </Button>
          </div>
        </Modal>
      ) : null}
    </>
  );
}
