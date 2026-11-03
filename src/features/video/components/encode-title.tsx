import { useTranslation } from 'react-i18next';
import type { TitleAnchor } from '@/bridge/task';
import { Select } from '@/components/select';
import { TextField } from '@/components/text-field';
import type { EncodeDraft } from '../encode-draft';
import styles from './encode.module.css';

type EncodeTitleProps = {
  draft: EncodeDraft;
  onChange: (draft: EncodeDraft) => void;
};

export function EncodeTitle({ draft, onChange }: EncodeTitleProps) {
  const { t } = useTranslation('video');
  function patch(next: Partial<EncodeDraft>) {
    onChange({ ...draft, ...next });
  }

  return (
    <div className={styles.fields}>
      <div className={styles.pair}>
        <Select
          label={t('encode.title.mode')}
          value={draft.titleMode}
          onChange={(event) => patch({ titleMode: event.target.value as EncodeDraft['titleMode'] })}
        >
          <option value="off">{t('encode.title.off')}</option>
          <option value="text">{t('encode.title.text')}</option>
          <option value="filename">{t('encode.title.filename')}</option>
        </Select>
        <Select
          label={t('encode.title.anchor')}
          value={draft.anchor}
          onChange={(event) => patch({ anchor: event.target.value as TitleAnchor })}
        >
          <option value="topStart">{t('encode.title.topStart')}</option>
          <option value="topCenter">{t('encode.title.topCenter')}</option>
          <option value="topEnd">{t('encode.title.topEnd')}</option>
          <option value="centerStart">{t('encode.title.centerStart')}</option>
          <option value="center">{t('encode.title.center')}</option>
          <option value="centerEnd">{t('encode.title.centerEnd')}</option>
          <option value="bottomStart">{t('encode.title.bottomStart')}</option>
          <option value="bottomCenter">{t('encode.title.bottomCenter')}</option>
          <option value="bottomEnd">{t('encode.title.bottomEnd')}</option>
        </Select>
      </div>
      {draft.titleMode === 'text' ? (
        <TextField
          label={t('encode.title.textLabel')}
          value={draft.titleText}
          onChange={(event) => patch({ titleText: event.target.value })}
        />
      ) : null}
      {draft.titleMode !== 'off' ? (
        <div className={styles.pair}>
          <TextField
            label={t('encode.title.fontSize')}
            value={draft.fontSize}
            inputMode="numeric"
            onChange={(event) => patch({ fontSize: event.target.value })}
          />
          <TextField
            label={t('encode.title.start')}
            value={draft.titleStart}
            inputMode="decimal"
            onChange={(event) => patch({ titleStart: event.target.value })}
          />
        </div>
      ) : null}
      {draft.titleMode !== 'off' ? (
        <TextField
          label={t('encode.title.end')}
          value={draft.titleEnd}
          inputMode="decimal"
          onChange={(event) => patch({ titleEnd: event.target.value })}
        />
      ) : null}
    </div>
  );
}
