import type { AvifDepth, AvifRange, AvifYuv } from '@/bridge/task';
import { Checkbox } from '@/components/checkbox';
import { Select } from '@/components/select';
import type { ImageParamsDraft } from '../params-draft';
import styles from './params.module.css';

type ParamsPixelProps = {
  draft: ImageParamsDraft;
  onChange: (draft: ImageParamsDraft) => void;
};

export function ParamsPixel({ draft, onChange }: ParamsPixelProps) {
  function patch(next: Partial<ImageParamsDraft>) {
    onChange({ ...draft, ...next });
  }

  return (
    <div className={styles.row}>
      <Select
        className={styles.choice}
        label="位深"
        value={draft.depth}
        onChange={(event) => patch({ depth: event.target.value as AvifDepth })}
      >
        <option value="eight">8-bit</option>
        <option value="ten">10-bit</option>
        <option value="twelve">12-bit</option>
      </Select>
      <Select
        className={styles.choice}
        label="色度"
        value={draft.yuv}
        onChange={(event) => patch({ yuv: event.target.value as AvifYuv })}
      >
        <option value="auto">自动</option>
        <option value="444">4:4:4</option>
        <option value="422">4:2:2</option>
        <option value="420">4:2:0</option>
        <option value="400">4:0:0</option>
      </Select>
      <Select
        className={styles.choice}
        label="范围"
        value={draft.range}
        onChange={(event) => patch({ range: event.target.value as AvifRange })}
      >
        <option value="auto">自动</option>
        <option value="full">全幅</option>
        <option value="limited">有限</option>
      </Select>
      <Checkbox
        label="SharpYUV"
        checked={draft.sharpyuv}
        onChange={(event) => patch({ sharpyuv: event.target.checked })}
      />
      <Checkbox
        label="预乘透明"
        checked={draft.premultiply}
        onChange={(event) => patch({ premultiply: event.target.checked })}
      />
    </div>
  );
}
