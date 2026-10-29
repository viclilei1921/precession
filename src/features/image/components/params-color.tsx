import { Select } from '@/components/select';
import { TextField } from '@/components/text-field';
import type { ImageParamsDraft } from '../params-draft';
import styles from './params.module.css';

type ParamsColorProps = {
  draft: ImageParamsDraft;
  onChange: (draft: ImageParamsDraft) => void;
};

export function ParamsColor({ draft, onChange }: ParamsColorProps) {
  function patch(next: Partial<ImageParamsDraft>) {
    onChange({ ...draft, ...next });
  }

  return (
    <div className={styles.row}>
      <Select
        className={styles.choiceWide}
        label="颜色编码"
        value={draft.colorEncoding}
        onChange={(event) => patch({ colorEncoding: event.target.value as ImageParamsDraft['colorEncoding'] })}
      >
        <option value="source">保持原样</option>
        <option value="srgb">sRGB</option>
        <option value="linearSrgb">线性 sRGB</option>
        <option value="srgbLuma">sRGB 灰度</option>
        <option value="linearSrgbLuma">线性灰度</option>
      </Select>
      <TextField
        className={styles.num}
        label="目标亮度"
        value={draft.targetIntensity}
        inputMode="decimal"
        placeholder="留空"
        onChange={(event) => patch({ targetIntensity: event.target.value })}
      />
    </div>
  );
}
