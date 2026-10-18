import type { BitDepth, Fit, HevcPreset, HevcTune, TitleAnchor, VideoContainer, Vp9Content } from '@/bridge/task';

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
    av1Bit: 'ten',
    hevcPreset: 'medium',
    hevcCrf: '26',
    hevcTune: 'none',
    hevcBit: 'ten',
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

const containerLabel: Record<VideoContainer, string> = {
  mp4: 'MP4',
  mov: 'MOV',
  mpegTs: 'MPEG-TS'
};

const bitLabel: Record<BitDepth, string> = {
  eight: '8-bit',
  ten: '10-bit'
};

/** 默认模式固定使用的编码。 */
export const defaultEncodeSummary = 'MP4 · AV1 · 速度 6 · CRF 35 · 10-bit';

/** 自定义模式当前会提交的画面参数。 */
export function encodeSummary(draft: EncodeDraft) {
  const container = draft.codec === 'vp9' && draft.container === 'mpegTs' ? 'MP4' : containerLabel[draft.container];
  if (draft.codec === 'hevc') {
    return `${container} · HEVC · ${draft.hevcPreset} · CRF ${draft.hevcCrf} · ${bitLabel[draft.hevcBit]}`;
  }
  if (draft.codec === 'vp9') {
    return `${container} · VP9 · 速度 ${draft.vp9Speed} · CRF ${draft.vp9Crf}`;
  }
  return `${container} · AV1 · 速度 ${draft.av1Preset} · CRF ${draft.av1Crf} · ${bitLabel[draft.av1Bit]}`;
}
