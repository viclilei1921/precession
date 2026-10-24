import { markRule } from '@milkdown/kit/prose';
import { toggleMark } from '@milkdown/kit/prose/commands';
import { $command, $inputRule, $markSchema, $remark } from '@milkdown/kit/utils';

type Phrasing = {
  type: string;
  value?: string;
  children?: Phrasing[];
};

type ToMarkdownState = {
  containerPhrasing: (node: Phrasing, info: { before: string; after: string }) => string;
};

function splitMarked(value: string): Phrasing[] | null {
  const pattern = /\+\+([\s\S]+?)\+\+|==([\s\S]+?)==/g;
  const parts: Phrasing[] = [];
  let cursor = 0;
  let found = false;
  for (const match of value.matchAll(pattern)) {
    const index = match.index ?? 0;
    found = true;
    if (index > cursor) {
      parts.push({ type: 'text', value: value.slice(cursor, index) });
    }
    const text = match[1] ?? match[2] ?? '';
    parts.push({
      type: match[1] != null ? 'underline' : 'highlight',
      children: [{ type: 'text', value: text }]
    });
    cursor = index + match[0].length;
  }
  if (!found) {
    return null;
  }
  if (cursor < value.length) {
    parts.push({ type: 'text', value: value.slice(cursor) });
  }
  return parts;
}

function rewrite(node: Phrasing) {
  if (node.type === 'code' || node.type === 'inlineCode' || !node.children) {
    return;
  }
  for (let index = node.children.length - 1; index >= 0; index -= 1) {
    const child = node.children[index];
    if (!child) {
      continue;
    }
    if (child.type === 'text' && child.value) {
      const parts = splitMarked(child.value);
      if (parts) {
        node.children.splice(index, 1, ...parts);
        continue;
      }
    }
    rewrite(child);
  }
}

function richMarks(this: { data: () => { toMarkdownExtensions?: unknown[] } }) {
  const data = this.data();
  data.toMarkdownExtensions ??= [];
  data.toMarkdownExtensions.push({
    handlers: {
      underline(node: Phrasing, _parent: unknown, state: ToMarkdownState) {
        return `++${state.containerPhrasing(node, { before: '+', after: '+' })}++`;
      },
      highlight(node: Phrasing, _parent: unknown, state: ToMarkdownState) {
        return `==${state.containerPhrasing(node, { before: '=', after: '=' })}==`;
      }
    }
  });
  return (tree: Phrasing) => {
    rewrite(tree);
  };
}

/** 让 `++下划线++` 和 `==高亮==` 能进出 Markdown */
export const richMarksRemark = $remark('rich-marks', () => richMarks);

function inlineMark(name: 'underline' | 'highlight', tag: 'u' | 'mark', marker: string) {
  const schema = $markSchema(name, () => ({
    parseDOM: [{ tag }],
    toDOM: () => [tag, {}],
    parseMarkdown: {
      match: (node: { type: string }) => node.type === name,
      runner: (state, node, markType) => {
        state.openMark(markType);
        state.next(node.children);
        state.closeMark(markType);
      }
    },
    toMarkdown: {
      match: (mark) => mark.type.name === name,
      runner: (state, mark) => {
        state.withMark(mark, name);
      }
    }
  }));
  const command = $command(`Toggle${name}`, (ctx) => () => toggleMark(schema.type(ctx)));
  const escaped = marker.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const input = $inputRule((ctx) => markRule(new RegExp(`(?<!\\w)(${escaped})(.+?)\\1$`), schema.type(ctx)));
  return { schema, command, input };
}

const underline = inlineMark('underline', 'u', '++');
const highlight = inlineMark('highlight', 'mark', '==');

export const underlineSchema = underline.schema;
export const toggleUnderlineCommand = underline.command;
export const underlineInputRule = underline.input;

export const highlightSchema = highlight.schema;
export const toggleHighlightCommand = highlight.command;
export const highlightInputRule = highlight.input;
