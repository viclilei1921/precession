import { useTranslation } from 'react-i18next';
import { Checkbox } from '@/components/checkbox';
import { TextField } from '@/components/text-field';
import type { ImageParamsDraft } from '../params-draft';
import styles from './params.module.css';

type ParamsFileProps = {
  draft: ImageParamsDraft;
  onChange: (draft: ImageParamsDraft) => void;
};

export function ParamsFile({ draft, onChange }: ParamsFileProps) {
  const { t } = useTranslation('image');

  function patch(next: Partial<ImageParamsDraft>) {
    onChange({ ...draft, ...next });
  }

  return (
    <div className={styles.row}>
      <TextField
        className={styles.num}
        label={t('params.labels.threads')}
        value={draft.jobs}
        inputMode="numeric"
        placeholder={t('params.placeholder.empty')}
        onChange={(event) => patch({ jobs: event.target.value })}
      />
      {draft.autotiling ? null : (
        <>
          <TextField
            className={styles.num}
            label={t('params.labels.tileRows')}
            value={draft.tilerowslog2}
            inputMode="numeric"
            placeholder={t('params.placeholder.tileRange')}
            onChange={(event) => patch({ tilerowslog2: event.target.value })}
          />
          <TextField
            className={styles.num}
            label={t('params.labels.tileCols')}
            value={draft.tilecolslog2}
            inputMode="numeric"
            placeholder={t('params.placeholder.tileRange')}
            onChange={(event) => patch({ tilecolslog2: event.target.value })}
          />
        </>
      )}
      <Checkbox
        label={t('params.labels.ignoreExif')}
        checked={draft.ignoreExif}
        onChange={(event) => patch({ ignoreExif: event.target.checked })}
      />
      <Checkbox
        label={t('params.labels.ignoreXmp')}
        checked={draft.ignoreXmp}
        onChange={(event) => patch({ ignoreXmp: event.target.checked })}
      />
      <Checkbox
        label={t('params.labels.ignoreIcc')}
        checked={draft.ignoreIcc}
        onChange={(event) => patch({ ignoreIcc: event.target.checked })}
      />
      <Checkbox
        label={t('params.labels.autoTiling')}
        checked={draft.autotiling}
        onChange={(event) => patch({ autotiling: event.target.checked })}
      />
      <Checkbox
        label={t('params.labels.progressive')}
        checked={draft.progressive}
        onChange={(event) => patch({ progressive: event.target.checked })}
      />
    </div>
  );
}
