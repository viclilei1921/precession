import type { Node } from '@milkdown/kit/prose/model';
import type { NodeViewConstructor } from '@milkdown/kit/prose/view';
import { $nodeSchema, $remark, $view } from '@milkdown/kit/utils';
import { mediaInfo, mediaSrc, uploadOpen } from '@/bridge';
import i18n from '@/i18n';
import { errorMessage } from '@/utils/error';

import { videoIdFromUrl } from './plain';

type Tree = {
  type: string;
  url?: string;
  children?: Tree[];
};

function videoNode(id: string): Tree {
  return { type: 'growth-video', url: id };
}

/** 把 `video:` 图片改成视频节点，避免被图片块拿去当图片画 */
function rewrite(node: Tree) {
  const children = node.children;
  if (!children) {
    return;
  }
  for (let index = children.length - 1; index >= 0; index -= 1) {
    const child = children[index];
    if (!child) {
      continue;
    }
    const blockId = child.type === 'image-block' ? videoIdFromUrl(child.url ?? '') : '';
    if (blockId) {
      children[index] = videoNode(blockId);
      continue;
    }
    if (
      child.type === 'paragraph' &&
      child.children?.some((item) => item.type === 'image' && videoIdFromUrl(item.url ?? ''))
    ) {
      const parts: Tree[] = [];
      let inline: Tree[] = [];
      for (const item of child.children) {
        const id = item.type === 'image' ? videoIdFromUrl(item.url ?? '') : '';
        if (id) {
          if (inline.length > 0) {
            parts.push({ type: 'paragraph', children: inline });
            inline = [];
          }
          parts.push(videoNode(id));
        } else {
          inline.push(item);
        }
      }
      if (inline.length > 0) {
        parts.push({ type: 'paragraph', children: inline });
      }
      children.splice(index, 1, ...parts);
      continue;
    }
    rewrite(child);
  }
}

export const growthVideoRemark = $remark('growth-video', () => () => rewrite);

function nodeId(node: Node) {
  const id = node.attrs.id;
  return typeof id === 'string' ? id : '';
}

export const growthVideoSchema = $nodeSchema('growth-video', () => ({
  inline: false,
  group: 'block',
  atom: true,
  selectable: true,
  draggable: false,
  isolating: true,
  marks: '',
  attrs: {
    id: { default: '', validate: 'string' }
  },
  parseDOM: [
    {
      tag: 'div[data-type="growth-video"]',
      getAttrs: (dom) => {
        if (!(dom instanceof HTMLElement)) {
          return false;
        }
        return { id: dom.getAttribute('data-id') ?? '' };
      }
    }
  ],
  toDOM: (node) => ['div', { 'data-type': 'growth-video', 'data-id': node.attrs.id, class: 'growth-video' }],
  parseMarkdown: {
    match: (node) => node.type === 'growth-video',
    runner: (state, node, type) => {
      const id = typeof node.url === 'string' ? node.url : '';
      if (!id) {
        return;
      }
      state.addNode(type, { id });
    }
  },
  toMarkdown: {
    match: (node) => node.type.name === 'growth-video',
    runner: (state, node) => {
      state.openNode('paragraph');
      state.addNode('image', undefined, undefined, {
        title: null,
        url: `video:${nodeId(node)}`,
        alt: ''
      });
      state.closeNode();
    }
  }
}));

/** Rust 端 `UploadError::PasswordRequired` 的文案由 `errorMessage` 翻成 `errors:auth.passwordRequired`，显示前先对齐。 */
function passwordRequiredText(): string {
  return i18n.t('errors:auth.passwordRequired');
}

const expandIcon = `
  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M15 3h6v6" /><path d="M9 21H3v-6" />
    <path d="M21 3l-7 7" /><path d="M3 21l7-7" />
  </svg>`;

function mountPlayer(dom: HTMLElement, id: string) {
  const note = document.createElement('p');
  note.className = 'growth-video-note';
  note.textContent = i18n.t('ui:markdown.videoPreparing');
  const video = document.createElement('video');
  video.className = 'growth-video-player';
  video.controls = true;
  video.playsInline = true;
  video.preload = 'metadata';
  video.hidden = true;
  const expand = document.createElement('button');
  expand.type = 'button';
  expand.className = 'growth-video-expand';
  expand.hidden = true;
  expand.innerHTML = expandIcon;
  const form = document.createElement('form');
  form.className = 'growth-video-form';
  form.hidden = true;
  const label = document.createElement('span');
  const input = document.createElement('input');
  input.className = 'growth-video-password';
  input.type = 'password';
  const button = document.createElement('button');
  button.className = 'growth-video-open';
  button.type = 'submit';
  const fail = document.createElement('span');
  fail.className = 'growth-video-fail';
  form.append(label, input, button, fail);
  dom.append(note, video, form);

  function expandVideo() {
    const src = video.currentSrc || video.src;
    if (!src) {
      return;
    }
    video.pause();
    window.dispatchEvent(new CustomEvent('precession:media-expand', { detail: { src } }));
  }
  expand.addEventListener('click', expandVideo);

  function applyLabels() {
    label.textContent = i18n.t('ui:markdown.videoPasswordPrompt');
    input.setAttribute('aria-label', i18n.t('ui:markdown.password'));
    button.textContent = i18n.t('ui:markdown.open');
    expand.setAttribute('aria-label', i18n.t('ui:markdown.videoExpand'));
  }
  applyLabels();
  i18n.on('languageChanged', applyLabels);

  let alive = true;

  async function open(password: string) {
    button.disabled = true;
    fail.textContent = '';
    note.hidden = false;
    note.textContent = i18n.t('ui:markdown.videoPreparing');
    form.hidden = true;
    video.hidden = true;
    try {
      await uploadOpen(id, password);
      const endpoint = await mediaInfo();
      if (!alive) {
        return;
      }
      const src = mediaSrc(endpoint, id);
      if (!src) {
        note.textContent = i18n.t('ui:markdown.videoUnavailable');
        return;
      }
      note.hidden = true;
      video.hidden = false;
      video.src = src;
      expand.hidden = false;
    } catch (error) {
      if (!alive) {
        return;
      }
      const text = errorMessage(error);
      const needsPassword = text === passwordRequiredText();
      if (needsPassword || password.length > 0) {
        note.hidden = true;
        form.hidden = false;
        fail.textContent = needsPassword ? '' : text;
        return;
      }
      note.textContent = text || i18n.t('ui:markdown.videoCannotOpen');
    } finally {
      if (alive) {
        button.disabled = false;
      }
    }
  }

  form.addEventListener('submit', (event) => {
    event.preventDefault();
    void open(input.value);
  });
  if (!id) {
    note.textContent = i18n.t('ui:markdown.videoMissing');
  } else {
    void open('');
  }

  return () => {
    alive = false;
    i18n.off('languageChanged', applyLabels);
    expand.removeEventListener('click', expandVideo);
    video.removeAttribute('src');
    video.load();
  };
}

export const growthVideoView = $view(growthVideoSchema.node, (): NodeViewConstructor => {
  return (node) => {
    const dom = document.createElement('div');
    dom.className = 'growth-video';
    dom.setAttribute('contenteditable', 'false');
    const stop = mountPlayer(dom, nodeId(node));
    return {
      dom,
      ignoreMutation: () => true,
      stopEvent: (event) => {
        const target = event.target;
        return target instanceof globalThis.Node && dom.contains(target);
      },
      selectNode: () => {
        dom.dataset.selected = 'true';
      },
      deselectNode: () => {
        delete dom.dataset.selected;
      },
      update: (next) => next.type === node.type && nodeId(next) === nodeId(node),
      destroy: stop
    };
  };
});
