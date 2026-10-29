import { toggleLinkCommand } from '@milkdown/kit/component/link-tooltip';
import type { CmdKey } from '@milkdown/kit/core';
import { commandsCtx, editorViewCtx } from '@milkdown/kit/core';
import type { Ctx } from '@milkdown/kit/ctx';
import {
  createCodeBlockCommand,
  insertHrCommand,
  toggleEmphasisCommand,
  toggleInlineCodeCommand,
  toggleStrongCommand,
  turnIntoTextCommand,
  wrapInBlockquoteCommand,
  wrapInBulletListCommand,
  wrapInHeadingCommand
} from '@milkdown/kit/preset/commonmark';
import { insertTableCommand, toggleStrikethroughCommand } from '@milkdown/kit/preset/gfm';
import {
  BracketsCurlyIcon,
  CodeBlockIcon,
  CodeIcon,
  EraserIcon,
  HighlighterIcon,
  ImageIcon,
  LinkIcon,
  ListBulletsIcon,
  MinusIcon,
  QuotesIcon,
  TableIcon,
  TextBIcon,
  TextHFourIcon,
  TextHOneIcon,
  TextHThreeIcon,
  TextHTwoIcon,
  TextItalicIcon,
  TextStrikethroughIcon,
  TextUnderlineIcon,
  VideoCameraIcon
} from '@phosphor-icons/react';
import type { ReactNode } from 'react';
import { IconButton } from '@/components/button/icon';
import styles from './index.module.css';
import { toggleHighlightCommand, toggleUnderlineCommand } from './marks';

type MarkdownToolbarProps = {
  disabled?: boolean;
  ready: boolean;
  source: boolean;
  busy: boolean;
  /** `null` 还没保存。不传则不显示图片 */
  image?: unknown;
  /** `null` 还没保存。不传则不显示视频 */
  video?: unknown;
  onPickImage: () => void;
  onPickVideo: () => void;
  onToggleSource: () => void;
  onAction: (run: (ctx: Ctx) => void) => void;
};

const headings = [
  { level: 1, label: '标题 1', icon: TextHOneIcon },
  { level: 2, label: '标题 2', icon: TextHTwoIcon },
  { level: 3, label: '标题 3', icon: TextHThreeIcon },
  { level: 4, label: '标题 4', icon: TextHFourIcon }
] as const;

export function MarkdownToolbar({
  disabled,
  ready,
  source,
  busy,
  image,
  video,
  onPickImage,
  onPickVideo,
  onToggleSource,
  onAction
}: MarkdownToolbarProps) {
  const locked = Boolean(disabled) || !ready || source;
  const call = <T,>(key: CmdKey<T>, payload?: T) => {
    onAction((ctx) => {
      ctx.get(commandsCtx).call(key, payload);
    });
  };

  return (
    <div className={styles.tools}>
      <div className={styles.bar}>
        <div className={styles.group}>
          {headings.map((item) => (
            <Tool
              key={item.level}
              label={item.label}
              disabled={locked}
              onClick={() => call(wrapInHeadingCommand.key, item.level)}
            >
              <item.icon />
            </Tool>
          ))}
          <Tool label="列表" disabled={locked} onClick={() => call(wrapInBulletListCommand.key)}>
            <ListBulletsIcon />
          </Tool>
          <Tool label="引用" disabled={locked} onClick={() => call(wrapInBlockquoteCommand.key)}>
            <QuotesIcon />
          </Tool>
        </div>
        <div className={styles.group}>
          <Tool label="加粗" disabled={locked} onClick={() => call(toggleStrongCommand.key)}>
            <TextBIcon />
          </Tool>
          <Tool label="斜体" disabled={locked} onClick={() => call(toggleEmphasisCommand.key)}>
            <TextItalicIcon />
          </Tool>
          <Tool label="下划线" disabled={locked} onClick={() => call(toggleUnderlineCommand.key)}>
            <TextUnderlineIcon />
          </Tool>
          <Tool label="代码" disabled={locked} onClick={() => call(toggleInlineCodeCommand.key)}>
            <CodeIcon />
          </Tool>
          <Tool label="删除线" disabled={locked} onClick={() => call(toggleStrikethroughCommand.key)}>
            <TextStrikethroughIcon />
          </Tool>
          <Tool label="高亮" disabled={locked} onClick={() => call(toggleHighlightCommand.key)}>
            <HighlighterIcon />
          </Tool>
          <Tool label="超链接" disabled={locked} onClick={() => call(toggleLinkCommand.key)}>
            <LinkIcon />
          </Tool>
          <Tool label="清空格式" disabled={locked} onClick={() => onAction(clearFormat)}>
            <EraserIcon />
          </Tool>
        </div>
        <div className={styles.group}>
          {image !== undefined ? (
            <Tool
              label={image === null ? '先保存，再插入图片' : '插入图片'}
              disabled={Boolean(disabled) || busy || image === null || (!source && !ready)}
              onClick={onPickImage}
            >
              <ImageIcon />
            </Tool>
          ) : null}
          {video !== undefined ? (
            <Tool
              label={video === null ? '先保存，再插入视频' : '插入视频'}
              disabled={Boolean(disabled) || busy || video === null || (!source && !ready)}
              onClick={onPickVideo}
            >
              <VideoCameraIcon />
            </Tool>
          ) : null}
          <Tool label="表格" disabled={locked} onClick={() => call(insertTableCommand.key, { row: 3, col: 3 })}>
            <TableIcon />
          </Tool>
          <Tool label="代码块" disabled={locked} onClick={() => call(createCodeBlockCommand.key)}>
            <CodeBlockIcon />
          </Tool>
          <Tool label="分割线" disabled={locked} onClick={() => call(insertHrCommand.key)}>
            <MinusIcon />
          </Tool>
        </div>
        <div className={styles.group}>
          <Tool
            label={source ? '排版' : '源码'}
            selected={source}
            disabled={Boolean(disabled)}
            onClick={onToggleSource}
          >
            <BracketsCurlyIcon />
          </Tool>
        </div>
      </div>
    </div>
  );
}

function Tool({
  label,
  disabled,
  selected,
  onClick,
  children
}: {
  label: string;
  disabled?: boolean;
  selected?: boolean;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <IconButton
      label={label}
      size="compact"
      shape="rounded"
      disabled={disabled}
      selected={selected}
      onMouseDown={(event) => event.preventDefault()}
      onClick={onClick}
    >
      {children}
    </IconButton>
  );
}

function clearFormat(ctx: Ctx) {
  const view = ctx.get(editorViewCtx);
  const { state, dispatch } = view;
  const { from, to, empty } = state.selection;
  if (empty) {
    dispatch(state.tr.setStoredMarks([]));
  } else {
    let tr = state.tr;
    for (const mark of Object.values(state.schema.marks)) {
      tr = tr.removeMark(from, to, mark);
    }
    dispatch(tr);
  }
  ctx.get(commandsCtx).call(turnIntoTextCommand.key);
}
