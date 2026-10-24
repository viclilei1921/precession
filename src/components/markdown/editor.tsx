import { Crepe } from '@milkdown/crepe';
import { insert } from '@milkdown/kit/utils';
import { Milkdown, MilkdownProvider, useEditor } from '@milkdown/react';
import type { CSSProperties, FocusEvent, Ref, RefObject } from 'react';
import { forwardRef, useEffect, useImperativeHandle, useRef, useState } from 'react';
import { onFileDrop, pickFiles } from '@/bridge';
import { Scroll } from '@/components/scroll';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import type { MarkdownImageOwner } from './image';
import { discardImages, imageExtension, importImageFile, importImagePath, uploadDisplayUrl } from './image';
import styles from './index.module.css';
import {
  highlightInputRule,
  highlightSchema,
  richMarksRemark,
  toggleHighlightCommand,
  toggleUnderlineCommand,
  underlineInputRule,
  underlineSchema
} from './marks';
import { markdownForEditor, markdownFromEditor } from './plain';
import { MarkdownToolbar } from './toolbar';

import '@milkdown/crepe/theme/common/style.css';
import '@milkdown/crepe/theme/frame.css';

const imageExts = ['png', 'jpg', 'jpeg', 'gif', 'webp', 'avif'];

/** 表格菜单默认朝上弹，贴着滚动区域上沿时会被裁掉。改放到空间更大的一侧。 */
function placeTableMenus(root: HTMLElement, viewport: HTMLElement) {
  const viewRect = viewport.getBoundingClientRect();
  for (const group of root.querySelectorAll<HTMLElement>('.milkdown-table-block .button-group')) {
    if (group.dataset.show !== 'true') {
      group.style.top = '';
      group.style.marginLeft = '';
      delete group.dataset.side;
      continue;
    }
    const handle = group.parentElement;
    if (!handle) {
      continue;
    }
    const height = group.offsetHeight;
    if (height === 0) {
      continue;
    }
    const handleRect = handle.getBoundingClientRect();
    const gap = 8;
    const spaceAbove = handleRect.top - viewRect.top;
    const spaceBelow = viewRect.bottom - handleRect.bottom;
    const openBelow = spaceAbove < height + gap && spaceBelow > spaceAbove;
    group.dataset.side = openBelow ? 'below' : 'above';
    group.style.top = openBelow ? `${handle.offsetHeight}px` : `${-(height + gap)}px`;
    group.style.marginLeft = '0px';
    const rect = group.getBoundingClientRect();
    let shift = 0;
    if (rect.left < viewRect.left + 4) {
      shift = viewRect.left + 4 - rect.left;
    } else if (rect.right > viewRect.right - 4) {
      shift = viewRect.right - 4 - rect.right;
    }
    group.style.marginLeft = shift === 0 ? '' : `${shift}px`;
  }
}

export type MarkdownEditorHandle = {
  saved: () => void;
};

type MarkdownEditorProps = {
  id?: string;
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  placeholder?: string;
  /** 编辑器整体高度，含工具栏。内容在下面单独滚动 */
  height?: string;
  compact?: boolean;
  /** 已有所属记录时传入。`null` 表示还没保存，插入按钮停用 */
  image?: MarkdownImageOwner | null;
  onFocusChange?: (focused: boolean) => void;
};

export const MarkdownEditor = forwardRef(function MarkdownEditor(
  { id, value, onChange, disabled, placeholder, height, compact, image, onFocusChange }: MarkdownEditorProps,
  ref: Ref<MarkdownEditorHandle>
) {
  const [source, setSource] = useState(false);
  const [ready, setReady] = useState(false);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState('');
  const crepeRef = useRef<Crepe | null>(null);
  const areaRef = useRef<HTMLTextAreaElement>(null);
  const rootRef = useRef<HTMLDivElement>(null);
  const viewportRef = useRef<HTMLDivElement>(null);
  const valueRef = useRef(value);
  const onChangeRef = useRef(onChange);
  const imageRef = useRef(image);
  const sourceRef = useRef(source);
  const focusedRef = useRef(false);
  const sessionRef = useRef(new Set<string>());
  const savedRef = useRef(false);
  const importPathsRef = useRef<(paths: string[]) => Promise<void>>(async () => {});
  const importFilesRef = useRef<(files: File[]) => Promise<void>>(async () => {});
  valueRef.current = value;
  onChangeRef.current = onChange;
  imageRef.current = image;
  sourceRef.current = source;

  useImperativeHandle(ref, () => ({
    saved() {
      savedRef.current = true;
      sessionRef.current.clear();
    }
  }));

  useEffect(() => {
    const session = sessionRef;
    const saved = savedRef;
    const currentImage = imageRef;
    return () => {
      if (saved.current || !currentImage.current) {
        return;
      }
      void discardImages(session.current);
    };
  }, []);

  useEffect(() => {
    if (source) {
      return;
    }
    const root = rootRef.current;
    const viewport = viewportRef.current;
    if (!root || !viewport) {
      return;
    }
    let frame = 0;
    const schedule = () => {
      if (frame !== 0) {
        return;
      }
      frame = requestAnimationFrame(() => {
        frame = 0;
        placeTableMenus(root, viewport);
      });
    };
    const keepMenu = (event: Event) => {
      const target = event.target;
      if (!(target instanceof Element)) {
        return;
      }
      const handle = target.closest('.cell-handle');
      const group = handle?.querySelector('.button-group');
      if (group?.getAttribute('data-show') !== 'true') {
        return;
      }
      event.stopPropagation();
    };
    const observer = new MutationObserver(schedule);
    observer.observe(root, { subtree: true, attributes: true, attributeFilter: ['data-show'] });
    root.addEventListener('pointermove', keepMenu, true);
    viewport.addEventListener('scroll', schedule, { passive: true });
    window.addEventListener('resize', schedule);
    return () => {
      observer.disconnect();
      root.removeEventListener('pointermove', keepMenu, true);
      viewport.removeEventListener('scroll', schedule);
      window.removeEventListener('resize', schedule);
      if (frame !== 0) {
        cancelAnimationFrame(frame);
      }
    };
  }, [source]);

  useEffect(() => {
    if (!image || disabled) {
      return;
    }
    let stop: (() => void) | undefined;
    let cancelled = false;
    void onFileDrop((event) => {
      if (event.phase !== 'drop' || !focusedRef.current) {
        return;
      }
      const paths = event.paths.filter((path) => imageExtension(path));
      if (paths.length > 0) {
        void importPathsRef.current(paths);
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
  }, [image, disabled]);

  useEffect(() => {
    const root = rootRef.current;
    if (!root || !image || disabled) {
      return;
    }
    const onPaste = (event: ClipboardEvent) => {
      const files = clipboardImages(event.clipboardData);
      if (files.length === 0) {
        return;
      }
      event.preventDefault();
      event.stopPropagation();
      void importFilesRef.current(files);
    };
    root.addEventListener('paste', onPaste, true);
    return () => root.removeEventListener('paste', onPaste, true);
  }, [image, disabled]);

  function rememberFocus(event: FocusEvent<HTMLDivElement>) {
    if (event.type === 'focus') {
      focusedRef.current = true;
      onFocusChange?.(true);
      if (event.target === event.currentTarget) {
        rootRef.current?.querySelector<HTMLElement>('[contenteditable="true"]')?.focus();
      }
      return;
    }
    const next = rootRef.current?.contains(event.relatedTarget as Node | null) ?? false;
    if (!next) {
      focusedRef.current = false;
      onFocusChange?.(false);
    }
  }

  function insertSnippet(id: string) {
    const snippet = `![](upload:${id})\n`;
    if (sourceRef.current) {
      const area = areaRef.current;
      const current = valueRef.current;
      const start = area?.selectionStart ?? current.length;
      const end = area?.selectionEnd ?? current.length;
      onChangeRef.current(current.slice(0, start) + snippet + current.slice(end));
      return;
    }
    const crepe = crepeRef.current;
    if (!crepe) {
      onChangeRef.current(`${valueRef.current}${snippet}`);
      return;
    }
    try {
      crepe.editor.action(insert(snippet));
    } catch {
      onChangeRef.current(`${valueRef.current}${snippet}`);
    }
  }

  async function importPaths(paths: string[]) {
    const owner = imageRef.current;
    if (!owner) {
      return;
    }
    setBusy(true);
    setNotice('');
    try {
      for (const path of paths) {
        const id = await importImagePath(path, owner, !useAppStore.getState().isMobile());
        sessionRef.current.add(id);
        insertSnippet(id);
      }
    } catch (error) {
      setNotice(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }
  importPathsRef.current = importPaths;

  async function importFiles(files: File[]) {
    const owner = imageRef.current;
    if (!owner) {
      return;
    }
    setBusy(true);
    setNotice('');
    try {
      for (const file of files) {
        const id = await importImageFile(file, owner, !useAppStore.getState().isMobile());
        sessionRef.current.add(id);
        insertSnippet(id);
      }
    } catch (error) {
      setNotice(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }
  importFilesRef.current = importFiles;

  async function acceptFile(file: File) {
    const owner = imageRef.current;
    if (!owner) {
      throw new Error('先保存，再插入图片');
    }
    const id = await importImageFile(file, owner, !useAppStore.getState().isMobile());
    sessionRef.current.add(id);
    return `upload:${id}`;
  }

  async function pick() {
    const paths = await pickFiles({ multiple: true, extensions: imageExts });
    if (paths.length > 0) {
      await importPaths(paths);
    }
  }

  function showSource() {
    try {
      const markdown = crepeRef.current?.getMarkdown();
      if (markdown != null) {
        const next = markdownFromEditor(markdown);
        if (next !== valueRef.current) {
          valueRef.current = next;
          onChangeRef.current(next);
        }
      }
    } catch {
      // 编辑器还没建好时，源码直接用当前字符串。
    }
    setSource(true);
  }

  return (
    <div
      ref={rootRef}
      id={source ? undefined : id}
      className={styles.editor}
      data-markdown=""
      style={{ '--markdown-height': height ?? (compact ? '8rem' : '18rem') } as CSSProperties}
      tabIndex={source ? undefined : -1}
      onFocusCapture={rememberFocus}
      onBlurCapture={rememberFocus}
    >
      <MarkdownToolbar
        disabled={disabled}
        ready={ready}
        source={source}
        busy={busy}
        image={image}
        onPickImage={() => void pick()}
        onToggleSource={() => {
          if (source) {
            setReady(false);
            setSource(false);
            return;
          }
          showSource();
        }}
        onAction={(run) => {
          crepeRef.current?.editor.action(run);
        }}
      />
      <div className={styles.body}>
        {source ? (
          <textarea
            ref={areaRef}
            id={id}
            className={styles.source}
            value={value}
            disabled={disabled}
            placeholder={placeholder}
            spellCheck={false}
            onChange={(event) => onChange(event.target.value)}
          />
        ) : (
          <Scroll className={styles.scroll} axis="vertical" viewportRef={viewportRef}>
            <div className={styles.surface}>
              <MilkdownProvider>
                <RichSurface
                  crepeRef={crepeRef}
                  valueRef={valueRef}
                  onChangeRef={onChangeRef}
                  imageRef={imageRef}
                  disabled={disabled}
                  placeholder={placeholder}
                  acceptFile={acceptFile}
                  onUploadError={setNotice}
                  onReady={() => setReady(true)}
                />
              </MilkdownProvider>
            </div>
          </Scroll>
        )}
      </div>
      {notice ? <p className={styles.notice}>{notice}</p> : null}
    </div>
  );
});

type RichSurfaceProps = {
  crepeRef: RefObject<Crepe | null>;
  valueRef: RefObject<string>;
  onChangeRef: RefObject<(value: string) => void>;
  imageRef: RefObject<MarkdownImageOwner | null | undefined>;
  disabled?: boolean;
  placeholder?: string;
  acceptFile: (file: File) => Promise<string>;
  onUploadError: (message: string) => void;
  onReady: () => void;
};

function RichSurface({
  crepeRef,
  valueRef,
  onChangeRef,
  imageRef,
  disabled,
  placeholder,
  acceptFile,
  onUploadError,
  onReady
}: RichSurfaceProps) {
  const acceptRef = useRef(acceptFile);
  const errorRef = useRef(onUploadError);
  const onReadyRef = useRef(onReady);
  acceptRef.current = acceptFile;
  errorRef.current = onUploadError;
  onReadyRef.current = onReady;

  const { loading } = useEditor((root) => {
    const upload = async (file: File) => {
      try {
        return await acceptRef.current(file);
      } catch (error) {
        errorRef.current(errorMessage(error));
        throw error;
      }
    };
    const crepe = new Crepe({
      root,
      defaultValue: markdownForEditor(valueRef.current ?? ''),
      features: {
        [Crepe.Feature.BlockEdit]: false,
        [Crepe.Feature.Latex]: false,
        [Crepe.Feature.Toolbar]: false
      },
      featureConfigs: {
        [Crepe.Feature.Placeholder]: {
          text: placeholder || ''
        },
        [Crepe.Feature.CodeMirror]: {
          /* 不用默认深色代码主题，颜色交给样式表，跟着应用的明暗走 */
          theme: []
        },
        [Crepe.Feature.ImageBlock]: {
          blockOnUpload: upload,
          inlineOnUpload: upload,
          onUpload: upload,
          blockCaptionPlaceholderText: '图片说明',
          proxyDomURL: (url) => {
            const owner = imageRef.current;
            if (!owner) {
              return url.startsWith('http://') || url.startsWith('https://') ? url : '';
            }
            return uploadDisplayUrl(url, owner);
          }
        }
      }
    });
    crepe.editor
      .use(richMarksRemark)
      .use(underlineSchema)
      .use(toggleUnderlineCommand)
      .use(underlineInputRule)
      .use(highlightSchema)
      .use(toggleHighlightCommand)
      .use(highlightInputRule);
    crepe.on((listener) => {
      listener.markdownUpdated((_ctx, markdown, prev) => {
        if (markdown === prev) {
          return;
        }
        const next = markdownFromEditor(markdown);
        if (next !== valueRef.current) {
          valueRef.current = next;
          onChangeRef.current?.(next);
        }
      });
    });
    crepeRef.current = crepe;
    return crepe;
  }, []);

  useEffect(
    () => () => {
      crepeRef.current = null;
    },
    [crepeRef]
  );

  useEffect(() => {
    if (loading) {
      return;
    }
    crepeRef.current?.setReadonly(Boolean(disabled));
    onReadyRef.current();
  }, [crepeRef, disabled, loading]);

  return <Milkdown />;
}

function clipboardImages(data: DataTransfer | null) {
  if (!data) {
    return [];
  }
  const listed = [...data.files].filter((file) => imageExtension(file.name) || file.type.startsWith('image/'));
  if (listed.length > 0) {
    return listed;
  }
  const files: File[] = [];
  for (const item of data.items) {
    if (item.kind !== 'file' || !item.type.startsWith('image/')) {
      continue;
    }
    const file = item.getAsFile();
    if (file) {
      files.push(file);
    }
  }
  return files;
}
