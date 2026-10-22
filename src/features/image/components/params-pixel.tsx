import type { AvifDepth, AvifRange, AvifYuv } from '@/bridge/task';
import { Checkbox } from '@/components/checkbox';
import { Select } from '@/components/select';
import type { ImageParamsDraft } from '../params-draft';
import { avifDepth, avifPremultiply, avifRange, avifSharpYuv, avifYuv } from '../params-notes';
import { HelpRow } from './help-row';
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
    <div className={styles.fields}>
      <div className={styles.pair}>
        <HelpRow label="位深说明" title={avifDepth}>
          <Select
            label="位深"
            value={draft.depth}
            onChange={(event) => patch({ depth: event.target.value as AvifDepth })}
          >
            <option value="eight">8-bit</option>
            <option value="ten">10-bit</option>
            <option value="twelve">12-bit</option>
          </Select>
        </HelpRow>
        <HelpRow label="色度说明" title={avifYuv}>
          <Select label="色度" value={draft.yuv} onChange={(event) => patch({ yuv: event.target.value as AvifYuv })}>
            <option value="auto">自动</option>
            <option value="444">4:4:4</option>
            <option value="422">4:2:2</option>
            <option value="420">4:2:0</option>
            <option value="400">4:0:0</option>
          </Select>
        </HelpRow>
      </div>
      <div className={styles.pair}>
        <HelpRow label="范围说明" title={avifRange}>
          <Select
            label="范围"
            value={draft.range}
            onChange={(event) => patch({ range: event.target.value as AvifRange })}
          >
            <option value="auto">自动</option>
            <option value="full">全幅</option>
            <option value="limited">有限</option>
          </Select>
        </HelpRow>
      </div>
      <div className={styles.checks}>
        <HelpRow align="check" label="SharpYUV 说明" title={avifSharpYuv}>
          <Checkbox
            label="SharpYUV"
            checked={draft.sharpyuv}
            onChange={(event) => patch({ sharpyuv: event.target.checked })}
          />
        </HelpRow>
        <HelpRow align="check" label="预乘透明说明" title={avifPremultiply}>
          <Checkbox
            label="预乘透明"
            checked={draft.premultiply}
            onChange={(event) => patch({ premultiply: event.target.checked })}
          />
        </HelpRow>
      </div>
    </div>
  );
}
