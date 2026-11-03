import { useTranslation } from 'react-i18next';
import type { AudioTrack } from '@/bridge/task';
import { Select } from '@/components/select';
import type { EncodeDraft } from '../encode-draft';
import styles from './encode.module.css';

type EncodeAudioProps = {
  draft: EncodeDraft;
  tracks: AudioTrack[];
  onChange: (draft: EncodeDraft) => void;
};

export function EncodeAudio({ draft, tracks, onChange }: EncodeAudioProps) {
  const { t } = useTranslation('video');
  function patch(next: Partial<EncodeDraft>) {
    onChange({ ...draft, ...next });
  }

  return (
    <div className={styles.fields}>
      <div className={styles.pair}>
        <Select
          label={t('encode.audio.track')}
          value={draft.audioKind}
          onChange={(event) => patch({ audioKind: event.target.value as EncodeDraft['audioKind'] })}
        >
          <option value="first">{t('encode.audio.first')}</option>
          <option value="index">{t('encode.audio.index')}</option>
          <option value="drop">{t('encode.audio.drop')}</option>
        </Select>
        <Select
          label={t('encode.audio.bitrate')}
          value={draft.bitrate}
          onChange={(event) => patch({ bitrate: event.target.value })}
        >
          <option value="96">96 kbps</option>
          <option value="128">128 kbps</option>
          <option value="160">160 kbps</option>
          <option value="192">192 kbps</option>
        </Select>
      </div>
      {draft.audioKind === 'index' ? (
        <Select
          label={t('encode.audio.trackIndex')}
          value={draft.audioIndex}
          onChange={(event) => patch({ audioIndex: event.target.value })}
        >
          <option value="">{t('encode.audio.chooseTrack')}</option>
          {tracks.map((track) => (
            <option key={track.index} value={String(track.index)}>
              {trackLabel(track, t)}
            </option>
          ))}
        </Select>
      ) : null}
      <div className={styles.pair}>
        <Select
          label={t('encode.audio.channels')}
          value={draft.channels}
          onChange={(event) => patch({ channels: event.target.value as EncodeDraft['channels'] })}
        >
          <option value="">{t('encode.audio.auto')}</option>
          <option value="1">{t('encode.audio.mono')}</option>
          <option value="2">{t('encode.audio.stereo')}</option>
        </Select>
        <Select
          label={t('encode.audio.sampleRate')}
          value={draft.sampleRate}
          onChange={(event) => patch({ sampleRate: event.target.value as EncodeDraft['sampleRate'] })}
        >
          <option value="">{t('encode.audio.auto')}</option>
          <option value="44100">44100</option>
          <option value="48000">48000</option>
        </Select>
      </div>
    </div>
  );
}

function trackLabel(track: AudioTrack, t: (key: string, options?: Record<string, unknown>) => string) {
  const language = track.language ? ` · ${track.language}` : '';
  const title = track.title ? ` · ${track.title}` : '';
  const channels = track.channelLayout || t('encode.audio.channels', { count: track.channels });
  return `${track.index} · ${track.codec} · ${channels}${language}${title}`;
}
