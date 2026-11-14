import type { Node as ProseNode } from '@milkdown/kit/prose/model';
import type { NodeViewConstructor } from '@milkdown/kit/prose/view';
import { $nodeSchema, $remark, $view } from '@milkdown/kit/utils';
import i18n from '@/i18n';

type Tree = {
  type: string;
  lang?: string | null;
  value?: string;
  children?: Tree[];
};

/** 把 ```mermaid 围栏改成图节点，交给 mermaid 画 */
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
    if (child.type === 'code' && child.lang === 'mermaid' && typeof child.value === 'string') {
      children[index] = { type: 'growth-mermaid', value: child.value };
      continue;
    }
    rewrite(child);
  }
}

export const growthMermaidRemark = $remark('growth-mermaid', () => () => rewrite);

export const growthMermaidSchema = $nodeSchema('growth-mermaid', () => ({
  inline: false,
  group: 'block',
  atom: true,
  selectable: true,
  draggable: false,
  isolating: true,
  marks: '',
  attrs: {
    code: { default: '', validate: 'string' }
  },
  parseDOM: [
    {
      tag: 'div[data-type="growth-mermaid"]',
      getAttrs: (dom) => {
        if (!(dom instanceof HTMLElement)) {
          return false;
        }
        return { code: dom.getAttribute('data-code') ?? '' };
      }
    }
  ],
  toDOM: (node) => ['div', { 'data-type': 'growth-mermaid', 'data-code': node.attrs.code, class: 'growth-mermaid' }],
  parseMarkdown: {
    match: (node) => node.type === 'growth-mermaid',
    runner: (state, node, type) => {
      state.addNode(type, { code: typeof node.value === 'string' ? node.value : '' });
    }
  },
  toMarkdown: {
    match: (node) => node.type.name === 'growth-mermaid',
    runner: (state, node) => {
      state.addNode('code', undefined, typeof node.attrs.code === 'string' ? node.attrs.code : '', {
        lang: 'mermaid'
      });
    }
  }
}));

type Mermaid = Awaited<typeof import('mermaid')>['default'];

let mermaidPromise: Promise<Mermaid> | null = null;

function loadMermaid() {
  mermaidPromise ??= import('mermaid').then((mod) => mod.default);
  return mermaidPromise;
}

function mermaidTheme() {
  return document.documentElement.classList.contains('dark') ? 'dark' : 'neutral';
}

function mermaidCode(node: ProseNode) {
  return typeof node.attrs.code === 'string' ? node.attrs.code : '';
}

export const growthMermaidView = $view(growthMermaidSchema.node, (): NodeViewConstructor => {
  return (node) => {
    const dom = document.createElement('div');
    dom.className = 'growth-mermaid';
    dom.setAttribute('contenteditable', 'false');
    const stage = document.createElement('div');
    stage.className = 'growth-mermaid-stage';
    const note = document.createElement('p');
    note.className = 'growth-mermaid-note';
    const source = document.createElement('pre');
    source.className = 'growth-mermaid-source';
    dom.append(stage, note, source);

    let alive = true;
    let seq = 0;
    let renderedTheme = '';

    function showCode(code: string, message: string) {
      stage.replaceChildren();
      note.hidden = false;
      note.textContent = message;
      source.hidden = false;
      source.textContent = code;
    }

    async function render(code: string) {
      note.hidden = true;
      source.hidden = true;
      stage.replaceChildren();
      try {
        const mermaid = await loadMermaid();
        if (!alive) {
          return;
        }
        mermaid.initialize({
          startOnLoad: false,
          securityLevel: 'strict',
          theme: mermaidTheme()
        });
        renderedTheme = mermaidTheme();
        const id = `growth-mermaid-${Date.now().toString(36)}-${seq + 1}`;
        seq += 1;
        const { svg } = await mermaid.render(id, code);
        if (!alive) {
          return;
        }
        stage.innerHTML = svg;
      } catch {
        if (!alive) {
          return;
        }
        showCode(code, i18n.t('ui:markdown.mermaidError'));
      }
    }

    void render(mermaidCode(node));

    // 跟随应用明暗：html.dark 变化时用新主题重画当前图
    const observer = new MutationObserver(() => {
      const theme = mermaidTheme();
      if (alive && theme !== renderedTheme) {
        void render(mermaidCode(node));
      }
    });
    observer.observe(document.documentElement, { attributeFilter: ['class'] });

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
      update: (next) => {
        if (next.type !== node.type) {
          return false;
        }
        const code = mermaidCode(next);
        if (code !== mermaidCode(node)) {
          void render(code);
        }
        return true;
      },
      destroy: () => {
        alive = false;
        observer.disconnect();
      }
    };
  };
});
