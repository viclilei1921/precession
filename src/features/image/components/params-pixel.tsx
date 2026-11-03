import { useTranslation } from 'react-i18next';
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
  const { t } = useTranslation('image');

  function patch(next: Partial<ImageParamsDraft>) {
    onChange({ ...draft, ...next });
  }

  return (
    <div className={styles.row}>
      <Select
        className={styles.choice}
        label={t('params.labels.depth')}
        value={draft.depth}
        onChange={(event) => patch({ depth: event.target.value as AvifDepth })}
      >
        <option value="eight">8-bit</option>
        <option value="ten">10-bit</option>
        <option value="twelve">12-bit</option>
      </Select>
      <Select
        className={styles.choice}
        label={t('params.labels.chroma')}
        value={draft.yuv}
        onChange={(event) => patch({ yuv: event.target.value as AvifYuv })}
      >
        <option value="auto">{t('params.options.auto')}</option>
        <option value="444">4:4:4</option>
        <option value="422">4:2:2</option>
        <option value="420">4:2:0</option>
        <option value="400">4:0:0</option>
      </Select>
      <Select
        className={styles.choice}
        label={t('params.labels.range')}
        value={draft.range}
        onChange={(event) => patch({ range: event.target.value as AvifRange })}
      >
        <option value="auto">{t('params.options.auto')}</option>
        <option value="full">{t('params.options.full')}</option>
        <option value="limited">{t('params.options.limited')}</option>
      </Select>
      <Checkbox
        label={t('params.labels.sharpYuv')}
        checked={draft.sharpyuv}
        onChange={(event) => patch({ sharpyuv: event.target.checked })}
      />
      <Checkbox
        label={t('params.labels.premultipliedAlpha')}
        checked={draft.premultiply}
        onChange={(event) => patch({ premultiply: event.target.checked })}
      />
    </div>
  );
}
