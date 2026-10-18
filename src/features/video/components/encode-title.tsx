import type { TitleAnchor } from '@/bridge/task';
import { Select } from '@/components/select';
import { TextField } from '@/components/text-field';
import type { EncodeDraft } from '../encode-draft';
import styles from './encode.module.css';

type EncodeTitleProps = {
  draft: EncodeDraft;
  onChange: (draft: EncodeDraft) => void;
};

export function EncodeTitle({ draft, onChange }: EncodeTitleProps) {
  function patch(next: Partial<EncodeDraft>) {
    onChange({ ...draft, ...next });
  }

  return (
    <div className={styles.fields}>
      <div className={styles.pair}>
        <Select
          label="标题"
          value={draft.titleMode}
          onChange={(event) => patch({ titleMode: event.target.value as EncodeDraft['titleMode'] })}
        >
          <option value="off">不画</option>
          <option value="text">自定义文字</option>
          <option value="filename">文件名</option>
        </Select>
        <Select
          label="标题位置"
          value={draft.anchor}
          onChange={(event) => patch({ anchor: event.target.value as TitleAnchor })}
        >
          <option value="topStart">左上</option>
          <option value="topCenter">上中</option>
          <option value="topEnd">右上</option>
          <option value="centerStart">左中</option>
          <option value="center">正中</option>
          <option value="centerEnd">右中</option>
          <option value="bottomStart">左下</option>
          <option value="bottomCenter">下中</option>
          <option value="bottomEnd">右下</option>
        </Select>
      </div>
      {draft.titleMode === 'text' ? (
        <TextField
          label="标题文字"
          value={draft.titleText}
          onChange={(event) => patch({ titleText: event.target.value })}
        />
      ) : null}
      {draft.titleMode !== 'off' ? (
        <div className={styles.pair}>
          <TextField
            label="字号"
            hint="留空为 24，按画面宽度缩放"
            value={draft.fontSize}
            inputMode="numeric"
            onChange={(event) => patch({ fontSize: event.target.value })}
          />
          <TextField
            label="标题开始（秒）"
            hint="留空从片头开始"
            value={draft.titleStart}
            inputMode="decimal"
            onChange={(event) => patch({ titleStart: event.target.value })}
          />
        </div>
      ) : null}
      {draft.titleMode !== 'off' ? (
        <TextField
          label="标题结束（秒）"
          hint="留空一直显示"
          value={draft.titleEnd}
          inputMode="decimal"
          onChange={(event) => patch({ titleEnd: event.target.value })}
        />
      ) : null}
    </div>
  );
}
