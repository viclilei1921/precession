import { useTranslation } from 'react-i18next';
import { Select } from '@/components/select';
import { TextField } from '@/components/text-field';
import type { ImageParamsDraft } from '../params-draft';
import styles from './params.module.css';

type ParamsColorProps = {
  draft: ImageParamsDraft;
  onChange: (draft: ImageParamsDraft) => void;
};

export function ParamsColor({ draft, onChange }: ParamsColorProps) {
  const { t } = useTranslation('image');

  function patch(next: Partial<ImageParamsDraft>) {
    onChange({ ...draft, ...next });
  }

  return (
    <div className={styles.row}>
      <Select
        className={styles.choiceWide}
        label={t('params.labels.colorEncoding')}
        value={draft.colorEncoding}
        onChange={(event) => patch({ colorEncoding: event.target.value as ImageParamsDraft['colorEncoding'] })}
      >
        <option value="source">{t('params.options.keepSource')}</option>
        <option value="srgb">sRGB</option>
        <option value="linearSrgb">{t('params.options.linearSrgb')}</option>
        <option value="srgbLuma">{t('params.options.srgbLuma')}</option>
        <option value="linearSrgbLuma">{t('params.options.linearSrgbLuma')}</option>
      </Select>
      <TextField
        className={styles.num}
        label={t('params.labels.targetIntensity')}
        value={draft.targetIntensity}
        inputMode="decimal"
        placeholder={t('params.placeholder.empty')}
        onChange={(event) => patch({ targetIntensity: event.target.value })}
      />
    </div>
  );
}
