import { useTranslation } from 'react-i18next';
import { Select } from '@/components/select';
import type { ImageParamsDraft } from '../params-draft';
import styles from './params.module.css';

type ParamsJxlPixelProps = {
  draft: ImageParamsDraft;
  onChange: (draft: ImageParamsDraft) => void;
};

export function ParamsJxlPixel({ draft, onChange }: ParamsJxlPixelProps) {
  const { t } = useTranslation('image');

  function patch(next: Partial<ImageParamsDraft>) {
    onChange({ ...draft, ...next });
  }

  return (
    <div className={styles.row}>
      <Select
        className={styles.choiceWide}
        label={t('params.labels.encodingMethod')}
        value={draft.jxlModular}
        onChange={(event) => patch({ jxlModular: event.target.value as ImageParamsDraft['jxlModular'] })}
      >
        <option value="auto">{t('params.options.auto')}</option>
        <option value="vardct">VarDCT</option>
        <option value="modular">{t('params.options.modular')}</option>
      </Select>
      <Select
        className={styles.choiceWide}
        label={t('params.labels.chromaSampling')}
        value={draft.jxlResampling}
        onChange={(event) => patch({ jxlResampling: event.target.value as ImageParamsDraft['jxlResampling'] })}
      >
        <option value="auto">{t('params.options.auto')}</option>
        <option value="1">{t('params.options.noDownscale')}</option>
        <option value="2">2×2</option>
        <option value="4">4×4</option>
        <option value="8">8×8</option>
      </Select>
      <Select
        className={styles.choiceWide}
        label={t('params.labels.premultipliedAlpha')}
        value={draft.jxlPremultiply}
        onChange={(event) => patch({ jxlPremultiply: event.target.value as ImageParamsDraft['jxlPremultiply'] })}
      >
        <option value="auto">{t('params.options.auto')}</option>
        <option value="off">{t('params.options.notPremultiplied')}</option>
        <option value="on">{t('params.options.premultiplied')}</option>
      </Select>
      <Select
        className={styles.choiceWide}
        label={t('params.labels.depth')}
        value={draft.jxlBitDepth}
        onChange={(event) => patch({ jxlBitDepth: event.target.value as ImageParamsDraft['jxlBitDepth'] })}
      >
        <option value="source">{t('params.options.followSource')}</option>
        <option value="8">8-bit</option>
        <option value="10">10-bit</option>
        <option value="12">12-bit</option>
        <option value="16">16-bit</option>
      </Select>
    </div>
  );
}
