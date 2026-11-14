import {
  CaretLeftIcon,
  CaretRightIcon,
  CornersInIcon,
  CornersOutIcon,
  FolderOpenIcon,
  ImageIcon,
  InfoIcon,
  ListIcon
} from '@phosphor-icons/react';
import type { CSSProperties } from 'react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { onFileDrop, pickDirectory, pickFiles } from '@/bridge/files';
import { isTauri, platform } from '@/bridge/platform';
import type { ViewerFile } from '@/bridge/viewer';
import {
  VIEWER_EXTENSIONS,
  viewerDirectSrc,
  viewerFolder,
  viewerRegisterDefault,
  viewerRename,
  viewerRetargetDisplay,
  viewerSrc
} from '@/bridge/viewer';
import { windowToggleFullScreen } from '@/bridge/window';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { Popover } from '@/components/popover';
import { Scroll } from '@/components/scroll';
import { Stage } from '@/components/stage';
import { createWheelPager } from '@/components/stage/wheel-paging';
import { TextField } from '@/components/text-field';
import { currentLocale } from '@/i18n';
import { useBackLayer } from '@/layout/mobile/back';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import { clearViewerPath, peekViewerPath, rememberViewerPath, retargetViewerPath, subscribeViewerPath } from './open';
import styles from './page.module.css';

const ZOOM_PRESETS = [25, 50, 100, 200, 400];

type Zoom = { kind: 'fit' } | { kind: 'fixed'; scale: number };
type Pixels = { width: number; height: number };

function onWindows() {
  if (!isTauri()) {
    return false;
  }
  try {
    return platform() === 'windows';
  } catch {
    return false;
  }
}

function formatName(extension: string) {
  if (extension === 'jpg' || extension === 'jpeg') {
    return 'JPEG';
  }
  if (extension === 'jxl') {
    return 'JPEG XL';
  }
  return extension.toUpperCase();
}

function formatBytes(bytes: number) {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${Math.round(bytes / 1024)} KB`;
  }
  const mb = bytes / (1024 * 1024);
  return mb >= 10 ? `${mb.toFixed(0)} MB` : `${mb.toFixed(1)} MB`;
}

function formatWhen(ms: number) {
  if (!ms) {
    return '—';
  }
  return new Date(ms).toLocaleString(currentLocale());
}

const RESERVED_NAME = /^(con|prn|aux|nul|com[1-9]|lpt[1-9])$/i;

function hasInvalidNameChar(value: string) {
  for (const char of value) {
    const code = char.codePointAt(0) ?? 0;
    if (code < 32 || '/\\:*?"<>|'.includes(char)) {
      return true;
    }
  }
  return false;
}

function asciiLower(value: string) {
  return value.replace(/[A-Z]/g, (letter) => String.fromCharCode(letter.charCodeAt(0) + 32));
}

/** 估算文案在输入框里的宽度（ch 单位）。中日韩等全角字符按 2 个算 */
function visualWidth(value: string) {
  let width = 0;
  for (const char of value) {
    width += (char.codePointAt(0) ?? 0) > 0x2e7f ? 2 : 1;
  }
  return width;
}

function compareText(left: string, right: string) {
  if (left < right) {
    return -1;
  }
  if (left > right) {
    return 1;
  }
  return 0;
}

/** 和后端同一层列表同一个顺序：先按忽略大小写，再按原名 */
function compareFiles(left: ViewerFile, right: ViewerFile) {
  const folded = compareText(asciiLower(left.name), asciiLower(right.name));
  if (folded !== 0) {
    return folded;
  }
  return compareText(left.name, right.name);
}

function splitFileName(name: string, extension: string) {
  const dot = `.${extension.toLowerCase()}`;
  if (extension && name.toLowerCase().endsWith(dot)) {
    const at = name.length - dot.length;
    return { stem: name.slice(0, at), suffix: name.slice(at) };
  }
  return { stem: name, suffix: '' };
}

function invalidStem(stem: string) {
  const head = stem.split('.')[0] ?? stem;
  if (!stem || stem === '.' || stem === '..') {
    return 'empty' as const;
  }
  if (hasInvalidNameChar(stem) || /[. ]$/.test(stem) || stem.length > 240 || RESERVED_NAME.test(head)) {
    return 'invalid' as const;
  }
  return null;
}

/** 系统模块里的本地看图页。 */
export function ViewerPage() {
  const { t } = useTranslation('viewer');
  const windows = onWindows();
  const [path, setPath] = useState(peekViewerPath);
  const [files, setFiles] = useState<ViewerFile[]>([]);
  const [index, setIndex] = useState(0);
  const [loading, setLoading] = useState(false);
  const [folderError, setFolderError] = useState('');
  const [src, setSrc] = useState<string | null>(null);
  const [srcFailed, setSrcFailed] = useState(false);
  const [natural, setNatural] = useState<Pixels | null>(null);
  const [zoom, setZoom] = useState<Zoom>({ kind: 'fit' });
  const [panReset, setPanReset] = useState(0);
  const [listOpen, setListOpen] = useState(false);
  const [full, setFull] = useState(false);
  const [detailsAnchor, setDetailsAnchor] = useState<HTMLButtonElement | null>(null);
  const [registerError, setRegisterError] = useState('');
  /** 换图时递增。改名只更换记住的路径，避免画面重载 */
  const [view, setView] = useState<{ path: string | null; token: number }>({ path: null, token: 0 });
  const fullRef = useRef(false);
  const leaveRef = useRef<() => void>(() => undefined);
  const chooseZoomRef = useRef<(value: string) => void>(() => undefined);
  const stageWrapRef = useRef<HTMLDivElement>(null);
  const zoomRef = useRef(zoom);
  zoomRef.current = zoom;
  const pager = useMemo(
    () =>
      createWheelPager((direction) => {
        setIndex((current) => {
          const total = filesRef.current.length;
          return direction > 0 ? Math.min(total - 1, current + 1) : Math.max(0, current - 1);
        });
      }),
    []
  );
  const filesRef = useRef(files);
  const indexRef = useRef(index);
  const pathRef = useRef(path);
  const skipReload = useRef<string | null>(null);
  const quietPath = useRef<string | null>(null);
  fullRef.current = full;
  filesRef.current = files;
  indexRef.current = index;
  pathRef.current = path;
  const file = files[index];
  const viewing = file != null;
  const fileRef = useRef(file);
  fileRef.current = file;
  const incoming = file?.path ?? null;
  if (quietPath.current != null && incoming != null && quietPath.current === incoming) {
    if (view.path !== incoming) {
      setView({ path: incoming, token: view.token });
      if (file && file.extension !== 'jxl') {
        setSrc(viewerDirectSrc(file.path));
      }
    }
  } else if (view.path !== incoming) {
    setView({ path: incoming, token: view.token + 1 });
    setZoom({ kind: 'fit' });
    setPanReset((current) => current + 1);
    setNatural(null);
    setSrcFailed(false);
    setSrc(file && file.extension !== 'jxl' ? viewerDirectSrc(file.path) : null);
  }

  useEffect(() => subscribeViewerPath(setPath), []);

  useEffect(() => {
    let stop: (() => void) | undefined;
    let cancelled = false;
    void onFileDrop((event) => {
      if (event.phase === 'drop' && event.paths[0]) {
        rememberViewerPath(event.paths[0]);
      }
    }).then((unlisten) => {
      if (cancelled) {
        unlisten();
        return;
      }
      stop = unlisten;
    });
    return () => {
      cancelled = true;
      stop?.();
    };
  }, []);

  useEffect(() => {
    if (!path) {
      setFiles([]);
      setFolderError('');
      setLoading(false);
      return;
    }
    if (skipReload.current === path) {
      skipReload.current = null;
      return;
    }
    let cancelled = false;
    setLoading(true);
    setFolderError('');
    setFiles([]);
    void viewerFolder(path)
      .then((folder) => {
        if (cancelled) {
          return;
        }
        setFiles(folder.files);
        setIndex(folder.index);
        setLoading(false);
      })
      .catch((error: unknown) => {
        if (cancelled) {
          return;
        }
        setFiles([]);
        setFolderError(errorMessage(error));
        setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [path]);

  useEffect(() => {
    if (quietPath.current != null && quietPath.current === file?.path) {
      quietPath.current = null;
    }
  }, [file?.path]);

  useEffect(() => {
    const current = fileRef.current;
    // 令牌只在换图时变。改名不进这里，避免把正在解码的图取消掉。
    if (view.token === 0 || current?.extension !== 'jxl') {
      return;
    }
    let cancelled = false;
    void viewerSrc(current)
      .then((next) => {
        if (!cancelled) {
          setSrc(next);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setSrcFailed(true);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [view.token]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const target = event.target;
      if (target instanceof HTMLElement && target.closest('input, textarea, select')) {
        return;
      }
      if (event.key === 'ArrowLeft') {
        event.preventDefault();
        setIndex((current) => Math.max(0, current - 1));
      } else if (event.key === 'ArrowRight') {
        event.preventDefault();
        setIndex((current) => Math.min(files.length - 1, current + 1));
      } else if (event.key === 'Escape' && path) {
        event.preventDefault();
        leaveRef.current();
      } else if (event.key === 'q' || event.key === 'Q') {
        event.preventDefault();
        leaveRef.current();
      } else if (event.key === '1') {
        event.preventDefault();
        chooseZoomRef.current('100');
      } else if (event.key === '0') {
        event.preventDefault();
        chooseZoomRef.current('fit');
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [files.length, path]);

  useEffect(() => {
    return () => {
      if (fullRef.current) {
        void windowToggleFullScreen(false);
      }
    };
  }, []);

  // 浏览图片时注册成一层返回：标题栏/系统返回先退出看图，再退路由
  useBackLayer(viewing, () => leaveRef.current());

  useEffect(() => {
    if (!viewing) {
      return;
    }
    const prior = useAppStore.getState().sidebarCollapsed;
    useAppStore.setState({ sidebarCollapsed: true });
    return () => {
      if (useAppStore.getState().sidebarCollapsed) {
        useAppStore.setState({ sidebarCollapsed: prior });
      }
    };
  }, [viewing]);

  /** 适应窗口模式下，上下滚轮翻上一张/下一张。放大后滚轮仍交给舞台平移。 */
  useEffect(() => {
    const node = stageWrapRef.current;
    if (!viewing || !node) {
      return;
    }
    function onWheel(event: WheelEvent) {
      if (zoomRef.current.kind !== 'fit') {
        return;
      }
      pager(event);
    }
    node.addEventListener('wheel', onWheel, { passive: true });
    return () => node.removeEventListener('wheel', onWheel);
  }, [pager, viewing]);

  async function openFile() {
    const picked = await pickFiles({ multiple: false, extensions: [...VIEWER_EXTENSIONS] });
    if (picked[0]) {
      rememberViewerPath(picked[0]);
    }
  }

  async function openFolder() {
    const directory = await pickDirectory();
    if (directory) {
      rememberViewerPath(directory);
    }
  }

  function onRenamed(from: string, next: ViewerFile) {
    const currentFiles = filesRef.current;
    const viewingFile = currentFiles[indexRef.current]?.path === from;
    if (viewingFile) {
      quietPath.current = next.path;
    }
    if (pathRef.current === from) {
      skipReload.current = next.path;
      setPath(next.path);
    }
    retargetViewerPath(from, next.path);
    viewerRetargetDisplay(from, next.path, next.extension);
    const replaced = currentFiles.map((item) => (item.path === from ? next : item));
    replaced.sort(compareFiles);
    setFiles(replaced);
    if (viewingFile) {
      const at = replaced.findIndex((item) => item.path === next.path);
      if (at >= 0) {
        setIndex(at);
      }
    }
  }

  function leave() {
    setFull(false);
    void windowToggleFullScreen(false).catch(() => undefined);
    clearViewerPath();
    setPath(null);
    setFiles([]);
    setIndex(0);
  }
  leaveRef.current = leave;
  chooseZoomRef.current = chooseZoom;

  function toggleFull() {
    const next = !full;
    setFull(next);
    void windowToggleFullScreen(next).catch(() => undefined);
  }

  async function register() {
    setRegisterError('');
    try {
      await viewerRegisterDefault();
    } catch (error: unknown) {
      setRegisterError(errorMessage(error));
    }
  }

  function chooseZoom(value: string) {
    setPanReset((current) => current + 1);
    if (value === 'fit') {
      setZoom({ kind: 'fit' });
      return;
    }
    setZoom({ kind: 'fixed', scale: Number(value) / 100 });
  }

  const percent = zoom.kind === 'fixed' ? Math.round(zoom.scale * 100) : null;
  const customPercent = percent != null && !ZOOM_PRESETS.includes(percent) ? percent : null;
  const width = file?.width || natural?.width || 0;
  const height = file?.height || natural?.height || 0;
  const sizeText = width > 0 && height > 0 ? `${width}×${height}` : t('bar.sizeUnknown');

  return (
    <div className={styles.page} data-full={full ? 'true' : undefined}>
      {file ? (
        <div ref={stageWrapRef} className={styles.stageWrap}>
          <div className={`${styles.edge} ${styles.prev}`}>
            <IconButton
              label={t('nav.prev')}
              disabled={index <= 0}
              onClick={() => setIndex((current) => Math.max(0, current - 1))}
            >
              <CaretLeftIcon weight="regular" />
            </IconButton>
          </div>
          {src ? (
            <Stage
              key={view.token}
              src={src}
              alt={file.name}
              scale={zoom.kind === 'fit' ? 'fit' : zoom.scale}
              panReset={panReset}
              onUserScale={(scale) => setZoom({ kind: 'fixed', scale })}
              onNatural={setNatural}
              onDoubleClick={() => chooseZoom(zoom.kind === 'fit' ? '100' : 'fit')}
            />
          ) : (
            <p className={styles.quiet}>
              {srcFailed ? t('stage.failed') : file.extension === 'jxl' ? t('stage.decoding') : t('empty.loading')}
            </p>
          )}
          <div className={`${styles.edge} ${styles.next}`}>
            <IconButton
              label={t('nav.next')}
              disabled={index >= files.length - 1}
              onClick={() => setIndex((current) => Math.min(files.length - 1, current + 1))}
            >
              <CaretRightIcon weight="regular" />
            </IconButton>
          </div>
          {file ? (
            <div className={styles.bar}>
              <div className={styles.cluster}>
                <IconButton
                  size="compact"
                  label={t('bar.list')}
                  selected={listOpen}
                  onClick={() => setListOpen((current) => !current)}
                >
                  <ListIcon weight="regular" />
                </IconButton>
                <p className={styles.meta}>
                  <span>{formatName(file.extension)}</span>
                  <span>{sizeText}</span>
                  <span>{formatBytes(file.bytes)}</span>
                </p>
              </div>
              <FileName file={file} onRenamed={onRenamed} />
              <div className={`${styles.cluster} ${styles.clusterEnd}`}>
                <IconButton
                  size="compact"
                  label={t('bar.details')}
                  expanded={detailsAnchor != null}
                  onClick={(event) => setDetailsAnchor((current) => (current ? null : event.currentTarget))}
                >
                  <InfoIcon weight="regular" />
                </IconButton>
                <select
                  className={styles.zoom}
                  aria-label={t('zoom.label')}
                  value={zoom.kind === 'fit' ? 'fit' : String(percent)}
                  onChange={(event) => chooseZoom(event.target.value)}
                >
                  <option value="fit">{t('zoom.fit')}</option>
                  {customPercent != null ? <option value={customPercent}>{customPercent}%</option> : null}
                  {ZOOM_PRESETS.map((preset) => (
                    <option key={preset} value={preset}>
                      {preset}%
                    </option>
                  ))}
                </select>
                <IconButton
                  size="compact"
                  label={t('bar.actual')}
                  className={styles.actual}
                  onClick={() => chooseZoom('100')}
                >
                  1:1
                </IconButton>
                <IconButton size="compact" label={full ? t('bar.exitFull') : t('bar.enterFull')} onClick={toggleFull}>
                  {full ? <CornersInIcon weight="regular" /> : <CornersOutIcon weight="regular" />}
                </IconButton>
                <Popover
                  open={detailsAnchor != null}
                  anchorEl={detailsAnchor}
                  label={t('bar.details')}
                  className={styles.details}
                  anchorOrigin={{ horizontal: 'right', vertical: 'top' }}
                  transformOrigin={{ horizontal: 'right', vertical: 'bottom' }}
                  gap={8}
                  onClose={() => setDetailsAnchor(null)}
                >
                  <dl>
                    <div>
                      <dt>{t('details.path')}</dt>
                      <dd>{file.path}</dd>
                    </div>
                    <div>
                      <dt>{t('details.modified')}</dt>
                      <dd>{formatWhen(file.modifiedMs)}</dd>
                    </div>
                    <div>
                      <dt>{t('details.format')}</dt>
                      <dd>{formatName(file.extension)}</dd>
                    </div>
                    <div>
                      <dt>{t('details.dimensions')}</dt>
                      <dd>{sizeText}</dd>
                    </div>
                    <div>
                      <dt>{t('details.bytes')}</dt>
                      <dd>{formatBytes(file.bytes)}</dd>
                    </div>
                  </dl>
                  {windows ? (
                    <Button variant="outlined" onClick={() => void register()}>
                      {t('details.register')}
                    </Button>
                  ) : null}
                  {registerError ? <p className={styles.notice}>{registerError}</p> : null}
                </Popover>
              </div>
            </div>
          ) : null}
        </div>
      ) : (
        <div className={styles.empty}>
          <h1>{t('empty.title')}</h1>
          <p className={styles.quiet}>
            {loading ? t('empty.loading') : folderError || (path ? t('empty.none') : t('empty.hint'))}
          </p>
          <div className={styles.actions}>
            <Button onClick={() => void openFile()}>
              <ImageIcon weight="regular" />
              {t('empty.openFile')}
            </Button>
            <Button variant="outlined" onClick={() => void openFolder()}>
              <FolderOpenIcon weight="regular" />
              {t('empty.openFolder')}
            </Button>
            {windows ? (
              <Button variant="text" onClick={() => void register()}>
                {t('details.register')}
              </Button>
            ) : null}
          </div>
          {registerError ? <p className={styles.notice}>{registerError}</p> : null}
        </div>
      )}

      {listOpen && files.length > 0 ? <Filmstrip files={files} index={index} onPick={setIndex} /> : null}
    </div>
  );
}

function FileName({ file, onRenamed }: { file: ViewerFile; onRenamed: (from: string, next: ViewerFile) => void }) {
  const { t } = useTranslation('viewer');
  const { stem, suffix } = splitFileName(file.name, file.extension);
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(stem);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const nameRef = useRef<HTMLDivElement>(null);
  const editingRef = useRef(false);
  const skipCommit = useRef(false);
  const ticket = useRef(0);

  useEffect(() => {
    // 同名文件换了路径也要退出编辑。主文件名不一定变。
    if (file.path.length === 0) {
      return;
    }
    ticket.current += 1;
    editingRef.current = false;
    setEditing(false);
    setError('');
    setDraft(stem);
  }, [stem, file.path]);

  useEffect(() => {
    if (!editing) {
      return;
    }
    const node = nameRef.current?.querySelector<HTMLInputElement>('input');
    node?.focus();
    node?.select();
  }, [editing]);

  function cancel() {
    skipCommit.current = true;
    editingRef.current = false;
    setEditing(false);
    setDraft(stem);
    setError('');
  }

  async function commit() {
    if (!editingRef.current || busy) {
      return;
    }
    const nextStem = draft.trim();
    if (nextStem === stem) {
      cancel();
      return;
    }
    const problem = invalidStem(nextStem);
    if (problem) {
      setError(t(problem === 'empty' ? 'name.empty' : 'name.invalid'));
      nameRef.current?.querySelector<HTMLInputElement>('input')?.focus();
      return;
    }
    const mine = ticket.current;
    const source = file.path;
    editingRef.current = false;
    setBusy(true);
    setError('');
    try {
      const updated = await viewerRename(source, `${nextStem}${suffix}`);
      onRenamed(source, updated);
      if (mine !== ticket.current) {
        return;
      }
      setEditing(false);
    } catch (caught: unknown) {
      if (mine !== ticket.current) {
        return;
      }
      editingRef.current = true;
      setError(errorMessage(caught));
      nameRef.current?.querySelector<HTMLInputElement>('input')?.focus();
    } finally {
      if (mine === ticket.current) {
        setBusy(false);
      }
    }
  }

  function begin() {
    setDraft(stem);
    setError('');
    editingRef.current = true;
    setEditing(true);
  }

  return (
    <div ref={nameRef} className={styles.name} data-editing={editing ? 'true' : undefined}>
      {editing ? (
        <form
          className={styles.nameForm}
          onSubmit={(event) => {
            event.preventDefault();
            void commit();
          }}
        >
          <TextField
            className={styles.nameField}
            label={t('name.edit')}
            error={error || undefined}
            value={draft}
            disabled={busy}
            spellCheck={false}
            autoCapitalize="off"
            autoCorrect="off"
            style={{ width: `${Math.min(Math.max(visualWidth(draft) + 3, 12), 48)}ch` }}
            onChange={(event) => {
              setDraft(event.target.value);
              setError('');
            }}
            onBlur={() => {
              if (skipCommit.current) {
                skipCommit.current = false;
                return;
              }
              void commit();
            }}
            onKeyDown={(event) => {
              if (event.key === 'Escape') {
                event.preventDefault();
                event.stopPropagation();
                cancel();
              }
            }}
          />
          {suffix ? <span className={styles.nameSuffix}>{suffix}</span> : null}
        </form>
      ) : (
        <button
          type="button"
          className={styles.nameButton}
          title={file.name}
          aria-label={`${t('name.edit')} ${file.name}`}
          onClick={begin}
        >
          {file.name}
        </button>
      )}
    </div>
  );
}

const THUMB_OVERSCAN = 4;
const THUMB_LOADS = 2;

let thumbLoads = 0;
const thumbWaiters: Array<() => void> = [];

function readCell(node: HTMLElement) {
  const cached = Number(node.dataset.cell);
  if (cached > 0) {
    return cached;
  }
  const raw = getComputedStyle(node).getPropertyValue('--thumb-cell').trim();
  const probe = document.createElement('div');
  probe.style.position = 'absolute';
  probe.style.visibility = 'hidden';
  probe.style.width = raw || '5rem';
  node.append(probe);
  const width = probe.getBoundingClientRect().width || 80;
  probe.remove();
  node.dataset.cell = String(width);
  return width;
}

function takeThumbLoad(): Promise<() => void> {
  return new Promise((resolve) => {
    const grant = () => {
      thumbLoads += 1;
      let released = false;
      resolve(() => {
        if (released) {
          return;
        }
        released = true;
        thumbLoads -= 1;
        thumbWaiters.shift()?.();
      });
    };
    if (thumbLoads < THUMB_LOADS) {
      grant();
      return;
    }
    thumbWaiters.push(grant);
  });
}

function Filmstrip({ files, index, onPick }: { files: ViewerFile[]; index: number; onPick: (index: number) => void }) {
  const scroller = useRef<HTMLDivElement>(null);
  const [range, setRange] = useState(() => ({
    start: Math.max(0, index - THUMB_OVERSCAN),
    end: Math.min(files.length, index + 8),
    cell: 80
  }));

  const measure = useCallback(() => {
    const node = scroller.current;
    if (!node) {
      return;
    }
    const cell = readCell(node);
    const start = Math.max(0, Math.floor(node.scrollLeft / cell) - THUMB_OVERSCAN);
    const visible = Math.ceil(node.clientWidth / cell) + THUMB_OVERSCAN * 2;
    const end = Math.min(files.length, start + Math.max(visible, 1));
    setRange((current) => {
      if (current.start === start && current.end === end && current.cell === cell) {
        return current;
      }
      return { start, end, cell };
    });
  }, [files.length]);

  useEffect(() => {
    const node = scroller.current;
    if (!node) {
      return;
    }
    const cell = readCell(node);
    const left = index * cell;
    const right = left + cell;
    if (left < node.scrollLeft || right > node.scrollLeft + node.clientWidth) {
      node.scrollLeft = Math.max(0, left - (node.clientWidth - cell) / 2);
    }
    measure();
  }, [index, measure]);

  useEffect(() => {
    const node = scroller.current;
    if (!node) {
      return;
    }
    measure();
    node.addEventListener('scroll', measure, { passive: true });
    const observer = new ResizeObserver(() => {
      delete node.dataset.cell;
      measure();
    });
    observer.observe(node);
    return () => {
      node.removeEventListener('scroll', measure);
      observer.disconnect();
    };
  }, [measure]);

  return (
    <Scroll className={styles.strip} axis="horizontal" viewportRef={scroller}>
      <div className={styles.track} style={{ width: files.length * range.cell }}>
        {files.slice(range.start, range.end).map((item, offset) => {
          const itemIndex = range.start + offset;
          return (
            <Thumb
              key={item.path}
              file={item}
              active={itemIndex === index}
              style={{ left: itemIndex * range.cell }}
              onPick={() => onPick(itemIndex)}
            />
          );
        })}
      </div>
    </Scroll>
  );
}

function Thumb({
  file,
  active,
  style,
  onPick
}: {
  file: ViewerFile;
  active: boolean;
  style: CSSProperties;
  onPick: () => void;
}) {
  const release = useRef<(() => void) | null>(null);
  const [src, setSrc] = useState<string | null>(null);
  const finish = useCallback(() => {
    release.current?.();
    release.current = null;
  }, []);

  useEffect(() => {
    let alive = true;
    void takeThumbLoad().then((done) => {
      if (!alive) {
        done();
        return;
      }
      release.current = done;
      void viewerSrc(file)
        .then((next) => {
          if (!alive) {
            finish();
            return;
          }
          setSrc(next);
        })
        .catch(() => finish());
    });
    return () => {
      alive = false;
      finish();
    };
  }, [file, finish]);

  return (
    <button
      type="button"
      className={styles.thumb}
      style={style}
      data-active={active ? 'true' : undefined}
      aria-current={active ? 'true' : undefined}
      aria-label={file.name}
      onClick={onPick}
    >
      {src ? <img src={src} alt="" decoding="async" onLoad={finish} onError={finish} /> : null}
    </button>
  );
}
