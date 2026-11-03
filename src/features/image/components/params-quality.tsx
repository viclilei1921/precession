import { useTranslation } from 'react-i18next';
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
  const { t } = useTranslation('image');

  function patch(next: Partial<ImageParamsDraft>) {
    onChange({ ...draft, ...next });
  }

  if (format === 'jxl') {
    return (
      <div className={styles.row}>
        <TextField
          className={styles.numShort}
          label={t('params.labels.distance')}
          value={draft.quality}
          inputMode="decimal"
          onChange={(event) => patch({ quality: event.target.value })}
        />
        <TextField
          className={styles.num}
          label={t('params.labels.effort')}
          value={draft.effort}
          inputMode="numeric"
          onChange={(event) => patch({ effort: event.target.value })}
        />
        <TextField
          className={styles.num}
          label={t('params.labels.decodingSpeed')}
          value={draft.decodingSpeed}
          inputMode="numeric"
          onChange={(event) => patch({ decodingSpeed: event.target.value })}
        />
        <TextField
          className={styles.num}
          label={t('params.labels.alphaDistance')}
          value={draft.alphaDistance}
          inputMode="decimal"
          placeholder={t('params.placeholder.empty')}
          onChange={(event) => patch({ alphaDistance: event.target.value })}
        />
        <TextField
          className={styles.num}
          label={t('params.labels.photonNoise')}
          value={draft.photonNoise}
          inputMode="numeric"
          placeholder={t('params.placeholder.empty')}
          onChange={(event) => patch({ photonNoise: event.target.value })}
        />
        <Checkbox
          label={t('params.labels.lossless')}
          checked={draft.lossless}
          onChange={(event) => patch({ lossless: event.target.checked })}
        />
        <Checkbox
          label={t('params.labels.losslessJpeg')}
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
        label={t('params.labels.colorQuality')}
        value={draft.qcolor}
        inputMode="numeric"
        onChange={(event) => patch({ qcolor: event.target.value })}
      />
      <TextField
        className={styles.num}
        label={t('params.labels.alphaQuality')}
        value={draft.qalpha}
        inputMode="numeric"
        onChange={(event) => patch({ qalpha: event.target.value })}
      />
      <TextField
        className={styles.numShort}
        label={t('params.labels.speed')}
        value={draft.speed}
        inputMode="numeric"
        onChange={(event) => patch({ speed: event.target.value })}
      />
      <TextField
        className={styles.num}
        label={t('params.labels.targetSize')}
        value={draft.targetSize}
        inputMode="numeric"
        placeholder={t('params.placeholder.empty')}
        onChange={(event) => patch({ targetSize: event.target.value })}
      />
      <Checkbox
        label={t('params.labels.lossless')}
        checked={draft.lossless}
        onChange={(event) => patch({ lossless: event.target.checked })}
      />
    </div>
  );
}
