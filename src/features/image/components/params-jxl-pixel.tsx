import { Select } from '@/components/select';
import type { ImageParamsDraft } from '../params-draft';
import { jxlBitDepth, jxlModular, jxlPremultiply, jxlResampling } from '../params-notes';
import { HelpRow } from './help-row';
import styles from './params.module.css';

type ParamsJxlPixelProps = {
  draft: ImageParamsDraft;
  onChange: (draft: ImageParamsDraft) => void;
};

export function ParamsJxlPixel({ draft, onChange }: ParamsJxlPixelProps) {
  function patch(next: Partial<ImageParamsDraft>) {
    onChange({ ...draft, ...next });
  }

  return (
    <div className={styles.fields}>
      <div className={styles.pair}>
        <HelpRow label="编码方式说明" title={jxlModular}>
          <Select
            label="编码方式"
            value={draft.jxlModular}
            onChange={(event) => patch({ jxlModular: event.target.value as ImageParamsDraft['jxlModular'] })}
          >
            <option value="auto">自动</option>
            <option value="vardct">VarDCT</option>
            <option value="modular">模块化</option>
          </Select>
        </HelpRow>
        <HelpRow label="色彩抽样说明" title={jxlResampling}>
          <Select
            label="色彩抽样"
            value={draft.jxlResampling}
            onChange={(event) => patch({ jxlResampling: event.target.value as ImageParamsDraft['jxlResampling'] })}
          >
            <option value="auto">自动</option>
            <option value="1">不缩小</option>
            <option value="2">2×2</option>
            <option value="4">4×4</option>
            <option value="8">8×8</option>
          </Select>
        </HelpRow>
      </div>
      <div className={styles.pair}>
        <HelpRow label="预乘透明说明" title={jxlPremultiply}>
          <Select
            label="预乘透明"
            value={draft.jxlPremultiply}
            onChange={(event) => patch({ jxlPremultiply: event.target.value as ImageParamsDraft['jxlPremultiply'] })}
          >
            <option value="auto">自动</option>
            <option value="off">不预乘</option>
            <option value="on">预乘</option>
          </Select>
        </HelpRow>
        <HelpRow label="位深说明" title={jxlBitDepth}>
          <Select
            label="位深"
            value={draft.jxlBitDepth}
            onChange={(event) => patch({ jxlBitDepth: event.target.value as ImageParamsDraft['jxlBitDepth'] })}
          >
            <option value="source">跟随原图</option>
            <option value="8">8-bit</option>
            <option value="10">10-bit</option>
            <option value="12">12-bit</option>
            <option value="16">16-bit</option>
          </Select>
        </HelpRow>
      </div>
    </div>
  );
}
