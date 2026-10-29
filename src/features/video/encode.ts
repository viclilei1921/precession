import type { VideoEncode } from '@/bridge/task';
import type { EncodeDraft } from './encode-draft';
import { initialEncodeDraft } from './encode-draft';

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

/** 把队列里记下的编码填回界面。和默认导出一致时回到默认模式。 */
export function formFromEncode(encode: VideoEncode | undefined): {
  mode: 'default' | 'custom';
  draft: EncodeDraft;
  drawFilename: boolean;
} {
  const draft = initialEncodeDraft();
  const video = encode?.video;
  if (video?.codec === 'hevc') {
    draft.codec = 'hevc';
    draft.hevcPreset = video.preset ?? 'medium';
    draft.hevcCrf = String(video.crf ?? 26);
    draft.hevcTune = video.tune ?? 'none';
    draft.hevcBit = video.bitDepth ?? 'ten';
  } else if (video?.codec === 'vp9') {
    draft.codec = 'vp9';
    draft.vp9Speed = String(video.cpuUsed ?? 2);
    draft.vp9Crf = String(video.crf ?? 33);
    draft.vp9Content = video.content ?? 'general';
  } else if (video?.codec === 'av1') {
    draft.av1Preset = String(video.preset ?? 6);
    draft.av1Crf = String(video.crf ?? 35);
    draft.av1Bit = video.bitDepth ?? 'ten';
  }
  if (encode?.container) {
    draft.container = encode.container;
  }
  if (encode?.fps) {
    draft.fps = String(encode.fps);
  }
  if (encode?.size) {
    draft.width = String(encode.size.width);
    draft.height = String(encode.size.height);
    draft.fit = encode.size.fit ?? 'contain';
  }
  const audio = encode?.audio;
  if (audio?.mode?.kind === 'index') {
    draft.audioKind = 'index';
    draft.audioIndex = String(audio.mode.index);
  } else if (audio?.mode?.kind === 'drop') {
    draft.audioKind = 'drop';
  }
  if (audio?.channels === 1 || audio?.channels === 2) {
    draft.channels = String(audio.channels) as '1' | '2';
  }
  if (audio?.sampleRate === 44100 || audio?.sampleRate === 48000) {
    draft.sampleRate = String(audio.sampleRate) as '44100' | '48000';
  }
  if (audio?.bitrateKbps) {
    draft.bitrate = String(audio.bitrateKbps);
  }
  if (encode?.title) {
    draft.titleMode = encode.title.source ?? 'text';
    draft.titleText = encode.title.text ?? '';
    draft.anchor = encode.title.anchor ?? 'topStart';
    draft.fontSize = encode.title.fontSize ? String(encode.title.fontSize) : '';
    draft.titleStart = encode.title.start == null ? '' : String(encode.title.start);
    draft.titleEnd = encode.title.end == null ? '' : String(encode.title.end);
  }
  if (encode?.subtitle?.source === 'embeddedTrack') {
    draft.subtitleMode = 'embeddedTrack';
    draft.subtitleTrack = encode.subtitle.trackIndex == null ? '' : String(encode.subtitle.trackIndex);
    draft.forceStyle = encode.subtitle.forceStyle ?? '';
  } else if (encode?.subtitle) {
    draft.subtitleMode = 'externalFile';
    draft.subtitleFile = encode.subtitle.file ?? '';
    draft.forceStyle = encode.subtitle.forceStyle ?? '';
  }
  const filenameOnly =
    encode?.title?.source === 'filename' &&
    !encode.title.text &&
    !encode.title.fontSize &&
    encode.title.start == null &&
    encode.title.end == null &&
    (encode.title.anchor == null || encode.title.anchor === 'topStart');
  const plain =
    (!encode?.container || encode.container === 'mp4') &&
    !encode?.fps &&
    !encode?.size &&
    !encode?.subtitle &&
    (!encode?.title || filenameOnly) &&
    (!video ||
      (video.codec === 'av1' &&
        (video.preset ?? 6) === 6 &&
        (video.crf ?? 35) === 35 &&
        (video.bitDepth ?? 'ten') === 'ten')) &&
    (!audio?.mode || audio.mode.kind === 'first') &&
    !audio?.channels &&
    !audio?.sampleRate &&
    (!audio?.bitrateKbps || audio.bitrateKbps === 128);
  return { mode: plain ? 'default' : 'custom', draft, drawFilename: Boolean(plain && filenameOnly) };
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
