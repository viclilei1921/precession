import { Crepe } from '@milkdown/crepe';
import { editorViewCtx } from '@milkdown/kit/core';
import { insert } from '@milkdown/kit/utils';
import { Milkdown, MilkdownProvider, useEditor } from '@milkdown/react';
import clsx from 'clsx';
import type { CSSProperties, FocusEvent, Ref, RefObject } from 'react';
import { forwardRef, useEffect, useImperativeHandle, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { onFileDrop, pickFiles } from '@/bridge';
import { Scroll } from '@/components/scroll';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import type { MarkdownImageOwner } from './image';
import {
  discardImages,
  imageExtension,
  importImageFile,
  importImagePath,
  uploadDisplayUrl,
  uploadIdFromUrl
} from './image';
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
import type { MediaItem } from './media-viewer';
import { MediaViewer } from './media-viewer';
import { growthMermaidRemark, growthMermaidSchema, growthMermaidView } from './mermaid-node';
import { imageIds, markdownForEditor, markdownFromEditor, videoIds } from './plain';
import { MarkdownToolbar } from './toolbar';
import { importVideoPath, videoExtension, videoExts } from './video';
import { growthVideoRemark, growthVideoSchema, growthVideoView } from './video-node';

import '@milkdown/crepe/theme/common/style.css';
import '@milkdown/crepe/theme/frame.css';

const imageExts = ['png', 'jpg', 'jpeg', 'gif', 'webp', 'avif'];

/** 微信朋友圈的格子规则：2 张各占一半，3 张一行三张，4 张田字格，5 张起三列流式排 */
function placeImageGrid(root: HTMLElement) {
  const parents = new Set<HTMLElement>();
  for (const el of root.querySelectorAll<HTMLElement>('.milkdown-image-block')) {
    if (el.parentElement) {
      parents.add(el.parentElement);
    }
  }
  for (const parent of parents) {
    let run: HTMLElement[] = [];
    const flush = () => {
      const size = run.length >= 5 ? '3' : run.length >= 2 ? String(run.length) : '';
      for (const el of run) {
        if (size) {
          el.dataset.galleryCells = size;
          // Crepe 在图片加载时会写内联高度，盖住格子的 cover 填充，这里清掉
          for (const img of el.querySelectorAll<HTMLImageElement>('img')) {
            img.style.removeProperty('height');
            img.style.removeProperty('max-width');
          }
        } else {
          delete el.dataset.galleryCells;
        }
      }
      run = [];
    };
    for (const child of parent.children) {
      if (child.classList.contains('milkdown-image-block')) {
        run.push(child as HTMLElement);
      } else {
        flush();
      }
    }
    flush();
  }
}

// 与项目图标一致（@phosphor-icons/react 线性风格）的替换/删除图标
const replaceIcon = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 1 1-3-6.7"/><path d="M21 3v6h-6"/></svg>`;
const deleteIcon = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/><path d="M10 11v6"/><path d="M14 11v6"/></svg>`;

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/** 编辑态给每张整段图片挂替换/删除按钮。Vue 组件的宿主上追加兄弟节点，不会被组件回收 */
function ensureImageActions(root: HTMLElement, labels: { replace: string; delete: string }) {
  for (const block of root.querySelectorAll<HTMLElement>('.milkdown-image-block')) {
    const getSrc = () => {
      const img = block.querySelector('img');
      return img?.currentSrc || img?.getAttribute('src') || img?.src || '';
    };
    if (!getSrc()) {
      continue;
    }
    if (!block.querySelector(':scope > .growth-image-replace')) {
      const button = document.createElement('button');
      button.type = 'button';
      button.className = 'growth-image-replace';
      button.setAttribute('contenteditable', 'false');
      button.setAttribute('aria-label', labels.replace);
      button.innerHTML = replaceIcon;
      button.addEventListener('click', (event) => {
        event.preventDefault();
        event.stopPropagation();
        const src = getSrc();
        if (src) {
          window.dispatchEvent(new CustomEvent('precession:image-replace', { detail: { src } }));
        }
      });
      block.appendChild(button);
    }
    if (!block.querySelector(':scope > .growth-image-delete')) {
      const button = document.createElement('button');
      button.type = 'button';
      button.className = 'growth-image-delete';
      button.setAttribute('contenteditable', 'false');
      button.setAttribute('aria-label', labels.delete);
      button.innerHTML = deleteIcon;
      button.addEventListener('click', (event) => {
        event.preventDefault();
        event.stopPropagation();
        const src = getSrc();
        if (src) {
          window.dispatchEvent(new CustomEvent('precession:image-delete', { detail: { src } }));
        }
      });
      block.appendChild(button);
    }
  }
}

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
  /** 记下这次保存。没留在正文里的新图片会被删掉。传入的正文优先于编辑器里的当前稿 */
  saved: (body?: string) => void;
  /** 当前正文。富文本从编辑器文档直接取出 */
  markdown: () => string;
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
  /** 和图片同一套归属。不传则不能插入视频 */
  video?: MarkdownImageOwner | null;
  /** 关闭后只留正文，阅读时用 */
  tools?: boolean;
  /** 滑块贴在右侧留白外缘，不盖住正文。父级右侧要有留白 */
  edge?: boolean;
  className?: string;
  onFocusChange?: (focused: boolean) => void;
};

export const MarkdownEditor = forwardRef(function MarkdownEditor(
  {
    id,
    value,
    onChange,
    disabled,
    placeholder,
    height,
    compact,
    image,
    video,
    tools = true,
    edge = false,
    className,
    onFocusChange
  }: MarkdownEditorProps,
  ref: Ref<MarkdownEditorHandle>
) {
  const { t } = useTranslation('ui');
  const [source, setSource] = useState(false);
  const [ready, setReady] = useState(false);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState('');
  const [viewer, setViewer] = useState<{ items: MediaItem[]; index: number } | null>(null);
  const crepeRef = useRef<Crepe | null>(null);
  const areaRef = useRef<HTMLTextAreaElement>(null);
  const rootRef = useRef<HTMLDivElement>(null);
  const viewportRef = useRef<HTMLDivElement>(null);
  const valueRef = useRef(value);
  const onChangeRef = useRef(onChange);
  const imageRef = useRef(image);
  const videoRef = useRef(video);
  const sourceRef = useRef(source);
  const focusedRef = useRef(false);
  const disabledRef = useRef(disabled);
  const labelsRef = useRef({ replace: t('markdown.imageReplace'), delete: t('markdown.imageDelete') });
  const replaceRef = useRef<(src: string) => Promise<void>>(async () => {});
  const removeRef = useRef<(src: string) => void>(() => undefined);
  const sessionRef = useRef(new Set<string>());
  const savedRef = useRef(false);
  const importPathsRef = useRef<(paths: string[]) => Promise<void>>(async () => {});
  const importFilesRef = useRef<(files: File[]) => Promise<void>>(async () => {});
  const importVideoPathsRef = useRef<(paths: string[]) => Promise<void>>(async () => {});
  valueRef.current = value;
  onChangeRef.current = onChange;
  imageRef.current = image;
  videoRef.current = video;
  sourceRef.current = source;
  disabledRef.current = Boolean(disabled);
  labelsRef.current = { replace: t('markdown.imageReplace'), delete: t('markdown.imageDelete') };

  function currentMarkdown() {
    if (sourceRef.current) {
      return valueRef.current;
    }
    try {
      const markdown = crepeRef.current?.getMarkdown();
      if (markdown != null) {
        return markdownFromEditor(markdown);
      }
    } catch {
      // 编辑器还没建好时，用当前字符串。
    }
    return valueRef.current;
  }

  useImperativeHandle(ref, () => ({
    saved(body?: string) {
      const markdown = body ?? currentMarkdown();
      const kept = new Set([...imageIds(markdown), ...videoIds(markdown)]);
      const drop = [...sessionRef.current].filter((id) => !kept.has(id));
      savedRef.current = true;
      sessionRef.current.clear();
      if (drop.length > 0) {
        void discardImages(drop);
      }
    },
    markdown() {
      return currentMarkdown();
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
        placeImageGrid(root);
        if (!disabledRef.current) {
          ensureImageActions(root, labelsRef.current);
        }
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
    observer.observe(root, { subtree: true, childList: true, attributes: true, attributeFilter: ['data-show'] });
    // 图片加载完成会触发 Crepe 写内联高度，不会产生 DOM 变更，单独调度一次
    const onLoad = (event: Event) => {
      if (event.target instanceof Element && event.target.closest('.milkdown-image-block')) {
        schedule();
      }
    };
    root.addEventListener('load', onLoad, true);
    schedule();
    root.addEventListener('pointermove', keepMenu, true);
    viewport.addEventListener('scroll', schedule, { passive: true });
    window.addEventListener('resize', schedule);
    return () => {
      observer.disconnect();
      root.removeEventListener('load', onLoad, true);
      root.removeEventListener('pointermove', keepMenu, true);
      viewport.removeEventListener('scroll', schedule);
      window.removeEventListener('resize', schedule);
      if (frame !== 0) {
        cancelAnimationFrame(frame);
      }
    };
  }, [source]);

  useEffect(() => {
    const root = rootRef.current;
    if (!root || disabledRef.current) {
      return;
    }
    const onExpand = (event: Event) => {
      const detail = (event as CustomEvent<{ src?: string }>).detail;
      if (!detail?.src) {
        return;
      }
      setViewer({ items: [{ kind: 'video', src: detail.src as string }], index: 0 });
    };
    window.addEventListener('precession:media-expand', onExpand);
    return () => window.removeEventListener('precession:media-expand', onExpand);
  }, []);

  useEffect(() => {
    const onReplace = (event: Event) => {
      const src = (event as CustomEvent<{ src?: string }>).detail?.src;
      if (src) {
        void replaceRef.current(src);
      }
    };
    window.addEventListener('precession:image-replace', onReplace);
    return () => window.removeEventListener('precession:image-replace', onReplace);
  }, []);

  useEffect(() => {
    const onDelete = (event: Event) => {
      const src = (event as CustomEvent<{ src?: string }>).detail?.src;
      if (src) {
        removeRef.current(src);
      }
    };
    window.addEventListener('precession:image-delete', onDelete);
    return () => window.removeEventListener('precession:image-delete', onDelete);
  }, []);

  useEffect(() => {
    const root = rootRef.current;
    if (!root) {
      return;
    }
    const openViewer = (image: HTMLImageElement) => {
      const media = [
        ...root.querySelectorAll<HTMLImageElement>('.milkdown-image-block img, .milkdown-image-inline img')
      ];
      const items: MediaItem[] = [];
      let index = 0;
      for (const node of media) {
        if (!node.src.startsWith('blob:') && !node.src.startsWith('http')) {
          continue;
        }
        if (node === image) {
          index = items.length;
        }
        items.push({ kind: 'image', src: node.src, caption: node.getAttribute('caption') || undefined });
      }
      if (items.length > 0) {
        setViewer({ items, index });
      }
    };
    const onOpen = (event: MouseEvent) => {
      const target = event.target;
      if (target instanceof HTMLImageElement && target.closest('.milkdown-image-block, .milkdown-image-inline')) {
        event.preventDefault();
        openViewer(target);
      }
    };
    root.addEventListener('dblclick', onOpen);
    return () => {
      root.removeEventListener('dblclick', onOpen);
    };
  }, []);

  useEffect(() => {
    if (disabled || (!image && !video)) {
      return;
    }
    let stop: (() => void) | undefined;
    let cancelled = false;
    void onFileDrop((event) => {
      if (event.phase !== 'drop' || !focusedRef.current) {
        return;
      }
      const images = image ? event.paths.filter((path) => imageExtension(path)) : [];
      const videos = video ? event.paths.filter((path) => videoExtension(path)) : [];
      if (images.length === 0 && videos.length === 0) {
        return;
      }
      void (async () => {
        if (images.length > 0) {
          await importPathsRef.current(images);
        }
        if (videos.length > 0) {
          await importVideoPathsRef.current(videos);
        }
      })();
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
  }, [image, video, disabled]);

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

  function insertSnippet(kind: 'upload' | 'video', id: string) {
    const snippet = `![](${kind}:${id})\n`;
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
        insertSnippet('upload', id);
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
        insertSnippet('upload', id);
      }
    } catch (error) {
      setNotice(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }
  importFilesRef.current = importFiles;

  async function importVideoPaths(paths: string[]) {
    const owner = videoRef.current;
    if (!owner) {
      return;
    }
    setBusy(true);
    setNotice('');
    try {
      for (const path of paths) {
        const id = await importVideoPath(path, owner);
        sessionRef.current.add(id);
        insertSnippet('video', id);
      }
    } catch (error) {
      setNotice(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }
  importVideoPathsRef.current = importVideoPaths;

  async function acceptFile(file: File) {
    const owner = imageRef.current;
    if (!owner) {
      throw new Error(t('errors:media.saveBeforeInsert'));
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

  async function pickVideo() {
    const paths = await pickFiles({ multiple: true, extensions: videoExts });
    if (paths.length > 0) {
      await importVideoPaths(paths);
    }
  }

  /** 替换某张图：导入新文件，把正文/文档里指向旧图的地址换掉 */
  async function replaceImage(src: string) {
    const owner = imageRef.current;
    if (!owner) {
      setNotice(t('errors:media.saveBeforeInsert'));
      return;
    }
    const paths = await pickFiles({ multiple: false, extensions: imageExts });
    if (paths.length === 0) {
      return;
    }
    setBusy(true);
    setNotice('');
    try {
      const id = await importImagePath(paths[0] ?? '', owner, !useAppStore.getState().isMobile());
      sessionRef.current.add(id);
      const oldId = uploadIdFromUrl(owner, src);
      const target = oldId ? `upload:${oldId}` : src;
      if (sourceRef.current) {
        const current = valueRef.current;
        if (current.includes(target)) {
          const next = current.replace(target, `upload:${id}`);
          valueRef.current = next;
          onChangeRef.current(next);
        } else {
          insertSnippet('upload', id);
        }
        return;
      }
      const crepe = crepeRef.current;
      if (!crepe) {
        insertSnippet('upload', id);
        return;
      }
      let done = false;
      try {
        crepe.editor.action((ctx) => {
          const view = ctx.get(editorViewCtx);
          view.state.doc.descendants((node, pos) => {
            if (done) {
              return false;
            }
            if ((node.type.name === 'image-block' || node.type.name === 'image') && node.attrs.src === target) {
              view.dispatch(view.state.tr.setNodeMarkup(pos, undefined, { ...node.attrs, src: `upload:${id}` }));
              done = true;
              return false;
            }
            return true;
          });
        });
      } catch {
        // 编辑器还没建好时，退回到追加插入
      }
      if (!done) {
        insertSnippet('upload', id);
      }
    } catch (error) {
      setNotice(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }

  replaceRef.current = replaceImage;

  /** 删除某张图：把正文里的节点/写法移除，文件本体交给保存时的清理逻辑回收 */
  function removeImage(src: string) {
    const owner = imageRef.current;
    const oldId = owner ? uploadIdFromUrl(owner, src) : null;
    const target = oldId ? `upload:${oldId}` : src;
    if (sourceRef.current) {
      const current = valueRef.current;
      const whole = new RegExp(`^!\\[[^\\]]*\\]\\(<?${escapeRegExp(target)}(?:\\s+"[^"]*")?>?\\)$`);
      const inline = new RegExp(`!\\[[^\\]]*\\]\\(<?${escapeRegExp(target)}(?:\\s+"[^"]*")?>?\\)`);
      const next = current
        .split('\n')
        .filter((line) => !whole.test(line.trim()))
        .join('\n')
        .replace(inline, '');
      if (next !== current) {
        valueRef.current = next;
        onChangeRef.current(next);
      }
      return;
    }
    const crepe = crepeRef.current;
    if (!crepe) {
      return;
    }
    try {
      crepe.editor.action((ctx) => {
        const view = ctx.get(editorViewCtx);
        let done = false;
        view.state.doc.descendants((node, pos) => {
          if (done) {
            return false;
          }
          if ((node.type.name === 'image-block' || node.type.name === 'image') && node.attrs.src === target) {
            view.dispatch(view.state.tr.delete(pos, pos + node.nodeSize));
            done = true;
            return false;
          }
          return true;
        });
      });
    } catch {
      // 编辑器还没建好时忽略
    }
  }

  removeRef.current = removeImage;

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
      className={clsx(styles.editor, className)}
      data-markdown=""
      style={{ '--markdown-height': height ?? (compact ? '8rem' : '18rem') } as CSSProperties}
      tabIndex={source ? undefined : -1}
      onFocusCapture={rememberFocus}
      onBlurCapture={rememberFocus}
    >
      {tools ? (
        <MarkdownToolbar
          disabled={disabled}
          ready={ready}
          source={source}
          busy={busy}
          image={image}
          video={video}
          onPickImage={() => void pick()}
          onPickVideo={() => void pickVideo()}
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
      ) : null}
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
          <Scroll className={styles.scroll} axis="vertical" edge={edge} viewportRef={viewportRef}>
            <div
              className={styles.surface}
              style={{ '--markdown-image-caption': JSON.stringify(t('markdown.imageCaption')) } as CSSProperties}
            >
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
      {viewer ? <MediaViewer items={viewer.items} index={viewer.index} onClose={() => setViewer(null)} /> : null}
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
  const { t } = useTranslation('ui');
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
          blockCaptionPlaceholderText: t('markdown.imageCaption'),
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
      .use(growthVideoSchema)
      .use(growthVideoRemark)
      .use(growthVideoView)
      .use(growthMermaidSchema)
      .use(growthMermaidRemark)
      .use(growthMermaidView)
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
