import { useTranslation } from 'react-i18next';
import { pickFiles } from '@/bridge/files';
import { Button } from '@/components/button';
import { Select } from '@/components/select';
import { TextField } from '@/components/text-field';
import type { EncodeDraft } from '../encode-draft';
import styles from './encode.module.css';

type EncodeSubtitleProps = {
  draft: EncodeDraft;
  onChange: (draft: EncodeDraft) => void;
};

export function EncodeSubtitle({ draft, onChange }: EncodeSubtitleProps) {
  const { t } = useTranslation('video');
  function patch(next: Partial<EncodeDraft>) {
    onChange({ ...draft, ...next });
  }

  return (
    <div className={styles.fields}>
      <Select
        label={t('encode.subtitle.mode')}
        value={draft.subtitleMode}
        onChange={(event) => patch({ subtitleMode: event.target.value as EncodeDraft['subtitleMode'] })}
      >
        <option value="off">{t('encode.subtitle.off')}</option>
        <option value="externalFile">{t('encode.subtitle.externalFile')}</option>
        <option value="embeddedTrack">{t('encode.subtitle.embeddedTrack')}</option>
      </Select>
      {draft.subtitleMode === 'externalFile' ? (
        <div className={styles.line}>
          <TextField
            label={t('encode.subtitle.file')}
            value={draft.subtitleFile}
            onChange={(event) => patch({ subtitleFile: event.target.value })}
          />
          <Button
            type="button"
            variant="text"
            onClick={() => {
              void pickFiles({ multiple: false, extensions: ['srt', 'ass', 'webvtt', 'vtt'] }).then((paths) => {
                const file = paths[0];
                if (file) {
                  patch({ subtitleFile: file });
                }
              });
            }}
          >
            {t('encode.subtitle.choose')}
          </Button>
        </div>
      ) : null}
      {draft.subtitleMode === 'embeddedTrack' ? (
        <TextField
          label={t('encode.subtitle.trackIndex')}
          value={draft.subtitleTrack}
          inputMode="numeric"
          onChange={(event) => patch({ subtitleTrack: event.target.value })}
        />
      ) : null}
      {draft.subtitleMode !== 'off' ? (
        <TextField
          label={t('encode.subtitle.style')}
          value={draft.forceStyle}
          onChange={(event) => patch({ forceStyle: event.target.value })}
        />
      ) : null}
    </div>
  );
}
