import { pickFiles } from '@/bridge/files';
import type {
  AudioTrack,
  BitDepth,
  Fit,
  HevcPreset,
  HevcTune,
  TitleAnchor,
  VideoContainer,
  VideoEncode,
  Vp9Content
} from '@/bridge/task';
import { Button } from '@/components/button';
import styles from '@/components/record.module.css';
import { Select } from '@/components/select';
import { TextField } from '@/components/text-field';

export type EncodeDraft = {
  container: VideoContainer;
  codec: 'av1' | 'hevc' | 'vp9';
  av1Preset: string;
  av1Crf: string;
  av1Bit: BitDepth;
  hevcPreset: HevcPreset;
  hevcCrf: string;
  hevcTune: HevcTune;
  hevcBit: BitDepth;
  vp9Speed: string;
  vp9Crf: string;
  vp9Content: Vp9Content;
  fps: string;
  width: string;
  height: string;
  fit: Fit;
  audioKind: 'first' | 'index' | 'drop';
  audioIndex: string;
  channels: '' | '1' | '2';
  sampleRate: '' | '44100' | '48000';
  bitrate: string;
  titleMode: 'off' | 'text' | 'filename';
  titleText: string;
  anchor: TitleAnchor;
  fontSize: string;
  titleStart: string;
  titleEnd: string;
  subtitleMode: 'off' | 'externalFile' | 'embeddedTrack';
  subtitleFile: string;
  subtitleTrack: string;
  forceStyle: string;
};

export function initialEncodeDraft(): EncodeDraft {
  return {
    container: 'mp4',
    codec: 'av1',
    av1Preset: '6',
    av1Crf: '35',
    av1Bit: 'eight',
    hevcPreset: 'medium',
    hevcCrf: '26',
    hevcTune: 'none',
    hevcBit: 'eight',
    vp9Speed: '2',
    vp9Crf: '33',
    vp9Content: 'general',
    fps: '',
    width: '',
    height: '',
    fit: 'contain',
    audioKind: 'first',
    audioIndex: '',
    channels: '',
    sampleRate: '',
    bitrate: '128',
    titleMode: 'off',
    titleText: '',
    anchor: 'topStart',
    fontSize: '',
    titleStart: '',
    titleEnd: '',
    subtitleMode: 'off',
    subtitleFile: '',
    subtitleTrack: '',
    forceStyle: ''
  };
}

function wholeNumber(value: string, fallback: number, max: number) {
  const parsed = Number(value.trim());
  if (!Number.isFinite(parsed)) {
    return fallback;
  }
  return Math.min(max, Math.max(0, Math.round(parsed)));
}

function optionalNumber(value: string) {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }
  const parsed = Number(trimmed);
  return Number.isFinite(parsed) ? parsed : null;
}

function positive(value: string) {
  const parsed = optionalNumber(value);
  return parsed != null && parsed > 0 ? Math.round(parsed) : null;
}

/** 自定义模式提交给后端的编码配置。空着的速度和质量用该编码的默认值。 */
export function buildEncode(draft: EncodeDraft, perFile: boolean): VideoEncode {
  const container = draft.codec === 'vp9' && draft.container === 'mpegTs' ? 'mp4' : draft.container;
  const video =
    draft.codec === 'hevc'
      ? {
          codec: 'hevc' as const,
          preset: draft.hevcPreset,
          crf: wholeNumber(draft.hevcCrf, 26, 51),
          tune: draft.hevcTune,
          bitDepth: draft.hevcBit
        }
      : draft.codec === 'vp9'
        ? {
            codec: 'vp9' as const,
            cpuUsed: wholeNumber(draft.vp9Speed, 2, 5),
            crf: wholeNumber(draft.vp9Crf, 33, 63),
            content: draft.vp9Content
          }
        : {
            codec: 'av1' as const,
            preset: wholeNumber(draft.av1Preset, 6, 13),
            crf: wholeNumber(draft.av1Crf, 35, 63),
            bitDepth: draft.av1Bit
          };
  const width = positive(draft.width);
  const height = positive(draft.height);
  const fps = positive(draft.fps);
  const encode: VideoEncode = {
    container,
    video,
    audio: {
      mode:
        draft.audioKind === 'index'
          ? { kind: 'index', index: wholeNumber(draft.audioIndex, 0, 64) }
          : { kind: draft.audioKind },
      channels: draft.channels === '' ? null : Number(draft.channels),
      sampleRate: draft.sampleRate === '' ? null : Number(draft.sampleRate),
      bitrateKbps: Math.min(320, Math.max(64, wholeNumber(draft.bitrate, 128, 320)))
    },
    faststart: true
  };
  if (perFile && fps) {
    encode.fps = fps;
  }
  if (width && height && draft.fit !== 'cover') {
    encode.size = { width, height, fit: draft.fit };
  }
  if (draft.fit === 'cover' && width && height) {
    encode.size = { width, height, fit: 'cover' };
  }
  if (draft.titleMode !== 'off') {
    encode.title = {
      source: draft.titleMode,
      text: draft.titleText.trim(),
      anchor: draft.anchor,
      fontSize: positive(draft.fontSize) ?? 0,
      start: optionalNumber(draft.titleStart),
      end: optionalNumber(draft.titleEnd)
    };
  }
  if (draft.subtitleMode === 'externalFile') {
    encode.subtitle = {
      source: 'externalFile',
      file: draft.subtitleFile.trim(),
      forceStyle: draft.forceStyle.trim() || null
    };
  }
  if (draft.subtitleMode === 'embeddedTrack') {
    encode.subtitle = {
      source: 'embeddedTrack',
      trackIndex: wholeNumber(draft.subtitleTrack, 0, 64),
      forceStyle: draft.forceStyle.trim() || null
    };
  }
  return encode;
}

type VideoEncodeFieldsProps = {
  draft: EncodeDraft;
  perFile: boolean;
  tracks: AudioTrack[];
  onChange: (draft: EncodeDraft) => void;
};

export function VideoEncodeFields({ draft, perFile, tracks, onChange }: VideoEncodeFieldsProps) {
  function patch(next: Partial<EncodeDraft>) {
    onChange({ ...draft, ...next });
  }

  const crfHint =
    draft.codec === 'av1'
      ? '0 最清晰，63 最小。默认 35'
      : draft.codec === 'hevc'
        ? '0 最清晰，51 最小。默认 26'
        : '0 最清晰，63 最小。默认 33';

  return (
    <>
      <div className={styles.pair}>
        <Select
          label="输出格式"
          value={draft.container}
          onChange={(event) => patch({ container: event.target.value as VideoContainer })}
        >
          <option value="mp4">MP4</option>
          <option value="mov">MOV</option>
          {draft.codec === 'vp9' ? null : <option value="mpegTs">MPEG-TS</option>}
        </Select>
        <Select
          label="视频编码"
          value={draft.codec}
          onChange={(event) => {
            const codec = event.target.value as EncodeDraft['codec'];
            patch({
              codec,
              container: codec === 'vp9' && draft.container === 'mpegTs' ? 'mp4' : draft.container
            });
          }}
        >
          <option value="av1">AV1</option>
          <option value="hevc">HEVC</option>
          <option value="vp9">VP9</option>
        </Select>
      </div>
      {draft.codec === 'av1' ? (
        <div className={styles.pair}>
          <Select
            label="速度"
            hint="数字越小越慢，同样质量下文件更小"
            value={draft.av1Preset}
            onChange={(event) => patch({ av1Preset: event.target.value })}
          >
            <option value="4">4 · 更小</option>
            <option value="6">6 · 默认</option>
            <option value="8">8 · 较快</option>
            <option value="10">10 · 很快</option>
          </Select>
          <TextField
            label="质量 CRF"
            hint={crfHint}
            value={draft.av1Crf}
            inputMode="numeric"
            onChange={(event) => patch({ av1Crf: event.target.value })}
          />
        </div>
      ) : null}
      {draft.codec === 'hevc' ? (
        <div className={styles.pair}>
          <Select
            label="速度"
            value={draft.hevcPreset}
            onChange={(event) => patch({ hevcPreset: event.target.value as HevcPreset })}
          >
            <option value="veryFast">veryfast · 很快</option>
            <option value="fast">fast · 较快</option>
            <option value="medium">medium · 默认</option>
            <option value="slow">slow · 更小</option>
            <option value="slower">slower · 更慢</option>
          </Select>
          <TextField
            label="质量 CRF"
            hint={crfHint}
            value={draft.hevcCrf}
            inputMode="numeric"
            onChange={(event) => patch({ hevcCrf: event.target.value })}
          />
        </div>
      ) : null}
      {draft.codec === 'vp9' ? (
        <div className={styles.pair}>
          <Select
            label="速度"
            hint="数字越小越慢"
            value={draft.vp9Speed}
            onChange={(event) => patch({ vp9Speed: event.target.value })}
          >
            <option value="1">1 · 更小</option>
            <option value="2">2 · 默认</option>
            <option value="4">4 · 较快</option>
            <option value="5">5 · 很快</option>
          </Select>
          <TextField
            label="质量 CRF"
            hint={crfHint}
            value={draft.vp9Crf}
            inputMode="numeric"
            onChange={(event) => patch({ vp9Crf: event.target.value })}
          />
        </div>
      ) : null}
      {draft.codec === 'av1' ? (
        <Select
          label="位深"
          hint="10-bit 渐变更干净，播放器支持更少"
          value={draft.av1Bit}
          onChange={(event) => patch({ av1Bit: event.target.value as BitDepth })}
        >
          <option value="eight">8-bit</option>
          <option value="ten">10-bit</option>
        </Select>
      ) : null}
      {draft.codec === 'hevc' ? (
        <div className={styles.pair}>
          <Select
            label="调优"
            value={draft.hevcTune}
            onChange={(event) => patch({ hevcTune: event.target.value as HevcTune })}
          >
            <option value="none">不调优</option>
            <option value="animation">动画</option>
            <option value="grain">保留颗粒</option>
          </Select>
          <Select
            label="位深"
            value={draft.hevcBit}
            onChange={(event) => patch({ hevcBit: event.target.value as BitDepth })}
          >
            <option value="eight">8-bit</option>
            <option value="ten">10-bit</option>
          </Select>
        </div>
      ) : null}
      {draft.codec === 'vp9' ? (
        <Select
          label="画面"
          value={draft.vp9Content}
          onChange={(event) => patch({ vp9Content: event.target.value as Vp9Content })}
        >
          <option value="general">普通画面</option>
          <option value="screen">录屏</option>
          <option value="film">胶片颗粒</option>
        </Select>
      ) : null}
      {perFile ? (
        <TextField
          label="帧率"
          hint="留空保留源帧率"
          value={draft.fps}
          inputMode="numeric"
          onChange={(event) => patch({ fps: event.target.value })}
        />
      ) : null}
      <div className={styles.pair}>
        <TextField
          label="宽度"
          hint="留空跟源。合并取最大，追加跟基准"
          value={draft.width}
          inputMode="numeric"
          onChange={(event) => patch({ width: event.target.value })}
        />
        <TextField
          label="高度"
          value={draft.height}
          inputMode="numeric"
          onChange={(event) => patch({ height: event.target.value })}
        />
      </div>
      <Select label="画面适配" value={draft.fit} onChange={(event) => patch({ fit: event.target.value as Fit })}>
        <option value="contain">缩进画面，不足处补边</option>
        <option value="fill">拉伸到目标尺寸</option>
        <option value="cover">铺满后裁切（当前不可用）</option>
      </Select>
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
      <div className={styles.pair}>
        <Select
          label="标题"
          value={draft.titleMode}
          onChange={(event) => patch({ titleMode: event.target.value as EncodeDraft['titleMode'] })}
        >
          <option value="off">不画</option>
          <option value="text">自定义文字</option>
          <option value="filename">文件名</option>
        </Select>
        <Select
          label="标题位置"
          value={draft.anchor}
          onChange={(event) => patch({ anchor: event.target.value as TitleAnchor })}
        >
          <option value="topStart">左上</option>
          <option value="topCenter">上中</option>
          <option value="topEnd">右上</option>
          <option value="centerStart">左中</option>
          <option value="center">正中</option>
          <option value="centerEnd">右中</option>
          <option value="bottomStart">左下</option>
          <option value="bottomCenter">下中</option>
          <option value="bottomEnd">右下</option>
        </Select>
      </div>
      {draft.titleMode === 'text' ? (
        <TextField
          label="标题文字"
          value={draft.titleText}
          onChange={(event) => patch({ titleText: event.target.value })}
        />
      ) : null}
      {draft.titleMode !== 'off' ? (
        <div className={styles.pair}>
          <TextField
            label="字号"
            hint="留空为 24，按画面宽度缩放"
            value={draft.fontSize}
            inputMode="numeric"
            onChange={(event) => patch({ fontSize: event.target.value })}
          />
          <TextField
            label="标题开始（秒）"
            hint="留空从片头开始"
            value={draft.titleStart}
            inputMode="decimal"
            onChange={(event) => patch({ titleStart: event.target.value })}
          />
        </div>
      ) : null}
      {draft.titleMode !== 'off' ? (
        <TextField
          label="标题结束（秒）"
          hint="留空一直显示"
          value={draft.titleEnd}
          inputMode="decimal"
          onChange={(event) => patch({ titleEnd: event.target.value })}
        />
      ) : null}
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
    </>
  );
}

function trackLabel(track: AudioTrack) {
  const language = track.language ? ` · ${track.language}` : '';
  const title = track.title ? ` · ${track.title}` : '';
  return `${track.index} · ${track.codec} · ${track.channelLayout || `${track.channels} 声道`}${language}${title}`;
}
