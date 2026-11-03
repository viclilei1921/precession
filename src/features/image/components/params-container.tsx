import { useTranslation } from 'react-i18next';
import { Checkbox } from '@/components/checkbox';
import { TextField } from '@/components/text-field';
import type { ImageParamsDraft } from '../params-draft';
import styles from './params.module.css';

type ParamsContainerProps = {
  draft: ImageParamsDraft;
  onChange: (draft: ImageParamsDraft) => void;
};

export function ParamsContainer({ draft, onChange }: ParamsContainerProps) {
  const { t } = useTranslation('image');

  function patch(next: Partial<ImageParamsDraft>) {
    onChange({ ...draft, ...next });
  }

  return (
    <div className={styles.row}>
      <TextField
        className={styles.num}
        label={t('params.labels.threads')}
        value={draft.jxlThreads}
        inputMode="numeric"
        placeholder={t('params.placeholder.empty')}
        onChange={(event) => patch({ jxlThreads: event.target.value })}
      />
      <Checkbox
        label={t('params.labels.useContainer')}
        checked={draft.useContainer}
        onChange={(event) => patch({ useContainer: event.target.checked })}
      />
      <Checkbox
        label={t('params.labels.progressive')}
        checked={draft.progressive}
        onChange={(event) => patch({ progressive: event.target.checked })}
      />
    </div>
  );
}
