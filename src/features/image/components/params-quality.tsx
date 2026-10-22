import { Checkbox } from '@/components/checkbox';
import { TextField } from '@/components/text-field';
import type { ImageParamsDraft } from '../params-draft';
import {
  avifAlpha,
  avifLossless,
  avifQuality,
  avifSpeed,
  avifTargetSize,
  jxlAlphaDistance,
  jxlDecoding,
  jxlDistance,
  jxlEffort,
  jxlLossless,
  jxlLosslessJpeg,
  jxlPhoton
} from '../params-notes';
import { HelpRow } from './help-row';
import styles from './params.module.css';

type ParamsQualityProps = {
  format: 'avif' | 'jxl';
  draft: ImageParamsDraft;
  onChange: (draft: ImageParamsDraft) => void;
};

export function ParamsQuality({ format, draft, onChange }: ParamsQualityProps) {
  function patch(next: Partial<ImageParamsDraft>) {
    onChange({ ...draft, ...next });
  }

  if (format === 'jxl') {
    return (
      <div className={styles.fields}>
        <div className={styles.pair}>
          <HelpRow label="距离说明" title={jxlDistance}>
            <TextField
              label="距离"
              hint="越小越清楚。建议 0.5 到 3，默认 1"
              value={draft.quality}
              inputMode="decimal"
              onChange={(event) => patch({ quality: event.target.value })}
            />
          </HelpRow>
          <HelpRow label="effort 说明" title={jxlEffort}>
            <TextField
              label="effort"
              hint="1 到 10，默认 7"
              value={draft.effort}
              inputMode="numeric"
              onChange={(event) => patch({ effort: event.target.value })}
            />
          </HelpRow>
        </div>
        <div className={styles.pair}>
          <HelpRow label="解码速度说明" title={jxlDecoding}>
            <TextField
              label="解码速度"
              hint="0 到 4，默认 0"
              value={draft.decodingSpeed}
              inputMode="numeric"
              onChange={(event) => patch({ decodingSpeed: event.target.value })}
            />
          </HelpRow>
          <HelpRow label="透明距离说明" title={jxlAlphaDistance}>
            <TextField
              label="透明距离"
              hint="留空则跟色彩距离"
              value={draft.alphaDistance}
              inputMode="decimal"
              onChange={(event) => patch({ alphaDistance: event.target.value })}
            />
          </HelpRow>
        </div>
        <div className={styles.pair}>
          <HelpRow label="光子噪声说明" title={jxlPhoton}>
            <TextField
              label="光子噪声"
              hint="ISO。留空则不加"
              value={draft.photonNoise}
              inputMode="numeric"
              onChange={(event) => patch({ photonNoise: event.target.value })}
            />
          </HelpRow>
        </div>
        <div className={styles.checks}>
          <HelpRow align="check" label="无损说明" title={jxlLossless}>
            <Checkbox
              label="无损"
              checked={draft.lossless}
              onChange={(event) => patch({ lossless: event.target.checked })}
            />
          </HelpRow>
          <HelpRow align="check" label="JPEG 无损重建说明" title={jxlLosslessJpeg}>
            <Checkbox
              label="JPEG 无损重建"
              checked={draft.losslessJpeg}
              onChange={(event) => patch({ losslessJpeg: event.target.checked })}
            />
          </HelpRow>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.fields}>
      <div className={styles.pair}>
        <HelpRow label="色彩质量说明" title={avifQuality}>
          <TextField
            label="色彩质量"
            hint="0 到 100，越大越清楚，默认 60"
            value={draft.qcolor}
            inputMode="numeric"
            onChange={(event) => patch({ qcolor: event.target.value })}
          />
        </HelpRow>
        <HelpRow label="透明质量说明" title={avifAlpha}>
          <TextField
            label="透明质量"
            hint="0 到 100，默认 60"
            value={draft.qalpha}
            inputMode="numeric"
            onChange={(event) => patch({ qalpha: event.target.value })}
          />
        </HelpRow>
      </div>
      <div className={styles.pair}>
        <HelpRow label="速度说明" title={avifSpeed}>
          <TextField
            label="速度"
            hint="0 最慢，10 最快，默认 6"
            value={draft.speed}
            inputMode="numeric"
            onChange={(event) => patch({ speed: event.target.value })}
          />
        </HelpRow>
        <HelpRow label="目标大小说明" title={avifTargetSize}>
          <TextField
            label="目标大小"
            hint="字节。留空则按质量"
            value={draft.targetSize}
            inputMode="numeric"
            onChange={(event) => patch({ targetSize: event.target.value })}
          />
        </HelpRow>
      </div>
      <HelpRow align="check" label="无损说明" title={avifLossless}>
        <Checkbox
          label="无损"
          checked={draft.lossless}
          onChange={(event) => patch({ lossless: event.target.checked })}
        />
      </HelpRow>
    </div>
  );
}
