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
  function patch(next: Partial<EncodeDraft>) {
    onChange({ ...draft, ...next });
  }

  return (
    <div className={styles.fields}>
      <Select
        label="字幕"
        value={draft.subtitleMode}
        onChange={(event) => patch({ subtitleMode: event.target.value as EncodeDraft['subtitleMode'] })}
      >
        <option value="off">不烧录</option>
        <option value="externalFile">外挂字幕</option>
        <option value="embeddedTrack">内嵌字幕轨</option>
      </Select>
      {draft.subtitleMode === 'externalFile' ? (
        <div className={styles.pair}>
          <TextField
            label="字幕文件"
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
            选择字幕
          </Button>
        </div>
      ) : null}
      {draft.subtitleMode === 'embeddedTrack' ? (
        <TextField
          label="字幕流序号"
          hint="Stream #0:N 的 N"
          value={draft.subtitleTrack}
          inputMode="numeric"
          onChange={(event) => patch({ subtitleTrack: event.target.value })}
        />
      ) : null}
      {draft.subtitleMode !== 'off' ? (
        <TextField
          label="字幕样式"
          hint="只对 SRT、WebVTT 生效，可以留空"
          value={draft.forceStyle}
          onChange={(event) => patch({ forceStyle: event.target.value })}
        />
      ) : null}
    </div>
  );
}
