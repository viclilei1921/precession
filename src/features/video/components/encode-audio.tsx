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
  function patch(next: Partial<EncodeDraft>) {
    onChange({ ...draft, ...next });
  }

  return (
    <div className={styles.fields}>
      <div className={styles.pair}>
        <Select
          label="音轨"
          value={draft.audioKind}
          onChange={(event) => patch({ audioKind: event.target.value as EncodeDraft['audioKind'] })}
        >
          <option value="first">第一条</option>
          <option value="index">指定一条</option>
          <option value="drop">不要音频</option>
        </Select>
        <Select label="音频码率" value={draft.bitrate} onChange={(event) => patch({ bitrate: event.target.value })}>
          <option value="96">96 kbps</option>
          <option value="128">128 kbps</option>
          <option value="160">160 kbps</option>
          <option value="192">192 kbps</option>
        </Select>
      </div>
      {draft.audioKind === 'index' ? (
        <Select
          label="音轨序号"
          value={draft.audioIndex}
          onChange={(event) => patch({ audioIndex: event.target.value })}
        >
          <option value="">选择音轨</option>
          {tracks.map((track) => (
            <option key={track.index} value={String(track.index)}>
              {trackLabel(track)}
            </option>
          ))}
        </Select>
      ) : null}
      <div className={styles.pair}>
        <Select
          label="声道"
          value={draft.channels}
          onChange={(event) => patch({ channels: event.target.value as EncodeDraft['channels'] })}
        >
          <option value="">自动</option>
          <option value="1">单声道</option>
          <option value="2">立体声</option>
        </Select>
        <Select
          label="采样率"
          value={draft.sampleRate}
          onChange={(event) => patch({ sampleRate: event.target.value as EncodeDraft['sampleRate'] })}
        >
          <option value="">自动</option>
          <option value="44100">44100</option>
          <option value="48000">48000</option>
        </Select>
      </div>
    </div>
  );
}

function trackLabel(track: AudioTrack) {
  const language = track.language ? ` · ${track.language}` : '';
  const title = track.title ? ` · ${track.title}` : '';
  return `${track.index} · ${track.codec} · ${track.channelLayout || `${track.channels} 声道`}${language}${title}`;
}
