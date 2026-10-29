import { Checkbox } from '@/components/checkbox';
import { TextField } from '@/components/text-field';
import type { ImageParamsDraft } from '../params-draft';
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
      <div className={styles.row}>
        <TextField
          className={styles.numShort}
          label="距离"
          value={draft.quality}
          inputMode="decimal"
          onChange={(event) => patch({ quality: event.target.value })}
        />
        <TextField
          className={styles.num}
          label="effort"
          value={draft.effort}
          inputMode="numeric"
          onChange={(event) => patch({ effort: event.target.value })}
        />
        <TextField
          className={styles.num}
          label="解码速度"
          value={draft.decodingSpeed}
          inputMode="numeric"
          onChange={(event) => patch({ decodingSpeed: event.target.value })}
        />
        <TextField
          className={styles.num}
          label="透明距离"
          value={draft.alphaDistance}
          inputMode="decimal"
          placeholder="留空"
          onChange={(event) => patch({ alphaDistance: event.target.value })}
        />
        <TextField
          className={styles.num}
          label="光子噪声"
          value={draft.photonNoise}
          inputMode="numeric"
          placeholder="留空"
          onChange={(event) => patch({ photonNoise: event.target.value })}
        />
        <Checkbox
          label="无损"
          checked={draft.lossless}
          onChange={(event) => patch({ lossless: event.target.checked })}
        />
        <Checkbox
          label="JPEG 无损重建"
          checked={draft.losslessJpeg}
          onChange={(event) => patch({ losslessJpeg: event.target.checked })}
        />
      </div>
    );
  }

  return (
    <div className={styles.row}>
      <TextField
        className={styles.num}
        label="色彩质量"
        value={draft.qcolor}
        inputMode="numeric"
        onChange={(event) => patch({ qcolor: event.target.value })}
      />
      <TextField
        className={styles.num}
        label="透明质量"
        value={draft.qalpha}
        inputMode="numeric"
        onChange={(event) => patch({ qalpha: event.target.value })}
      />
      <TextField
        className={styles.numShort}
        label="速度"
        value={draft.speed}
        inputMode="numeric"
        onChange={(event) => patch({ speed: event.target.value })}
      />
      <TextField
        className={styles.num}
        label="目标大小"
        value={draft.targetSize}
        inputMode="numeric"
        placeholder="留空"
        onChange={(event) => patch({ targetSize: event.target.value })}
      />
      <Checkbox label="无损" checked={draft.lossless} onChange={(event) => patch({ lossless: event.target.checked })} />
    </div>
  );
}
