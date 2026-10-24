import type { UnlistenFn } from '@tauri-apps/api/event';
import { listen } from '@tauri-apps/api/event';

import { invokeCommand } from './invoke';
import type { OwnerKind, UploadKind } from './upload';

/** 任务状态 */
export type TaskStatus = 'pending' | 'processing' | 'completed' | 'failed' | 'canceled';

/** 队列里的一条任务 */
export type TaskKind =
  | 'encryptFile'
  | 'decryptFile'
  | 'importUpload'
  | 'encryptUpload'
  | 'decryptUpload'
  | 'convertVideo'
  | 'cutVideo'
  | 'mergeVideo'
  | 'appendVideo'
  | 'convertAvif'
  | 'convertJxl';

/** 失败或取消后还原界面用。不含密码。 */
export type TaskDraft =
  | { kind: 'encryptFile'; input: string; output: string }
  | { kind: 'decryptFile'; input: string; output: string }
  | { kind: 'convertVideo'; input: string; output: string; encode: VideoEncode }
  | { kind: 'cutVideo'; input: string; output: string; segments: TimeSegment[]; encode: VideoEncode }
  | { kind: 'mergeVideo'; inputs: string[]; output: string; encode: VideoEncode }
  | { kind: 'appendVideo'; base: string; inputs: string[]; output: string; encode: VideoEncode }
  | { kind: 'convertAvif'; input: string; output: string; params: AvifEncodeParams }
  | { kind: 'convertJxl'; input: string; output: string; params: JxlEncodeParams };

/** 队列里的一条任务 */
export type Task = {
  id: string;
  kind: TaskKind;
  status: TaskStatus;
  progress: number;
  message: string;
  uploadId: string | null;
  /** 文件名。多文件时带上数量。 */
  title: string;
  /** 输出文件。空字符串表示没有可打开的结果。 */
  output: string;
  draft: TaskDraft | null;
};

/** 裁剪片段。start / duration 为 `HH:MM:SS` 或秒数。 */
export type TimeSegment = {
  start: string;
  duration: string;
};

/** 写出容器。侧车只打开了这三种。 */
export type VideoContainer = 'mp4' | 'mov' | 'mpegTs';

/** 位深。只有 AV1 和 HEVC 有这一项。 */
export type BitDepth = 'eight' | 'ten';

export type HevcPreset = 'veryFast' | 'fast' | 'medium' | 'slow' | 'slower';

export type HevcTune = 'none' | 'animation' | 'grain';

export type Vp9Content = 'general' | 'screen' | 'film';

export type Fit = 'contain' | 'fill' | 'cover';

export type TitleSource = 'text' | 'filename';

export type TitleAnchor =
  | 'topStart'
  | 'topCenter'
  | 'topEnd'
  | 'centerStart'
  | 'center'
  | 'centerEnd'
  | 'bottomStart'
  | 'bottomCenter'
  | 'bottomEnd';

export type SubtitleSource = 'externalFile' | 'embeddedTrack';

/** 当前编码和它自己的速度、质量。缺省是 AV1，preset 6，CRF 35。 */
export type VideoParams =
  | { codec: 'av1'; preset?: number; crf?: number; bitDepth?: BitDepth }
  | { codec: 'hevc'; preset?: HevcPreset; crf?: number; tune?: HevcTune; bitDepth?: BitDepth }
  | { codec: 'vp9'; cpuUsed?: number; crf?: number; content?: Vp9Content };

export type AudioMode = { kind: 'first' } | { kind: 'index'; index: number } | { kind: 'drop' };

export type AudioSelection = {
  mode?: AudioMode;
  channels?: number | null;
  sampleRate?: number | null;
  bitrateKbps?: number;
};

export type VideoSize = {
  width: number;
  height: number;
  fit?: Fit;
};

export type TitleOverlay = {
  source?: TitleSource;
  text?: string;
  anchor?: TitleAnchor;
  fontSize?: number;
  fontFile?: string | null;
  start?: number | null;
  end?: number | null;
};

export type SubtitleOverlay = {
  source?: SubtitleSource;
  file?: string | null;
  trackIndex?: number | null;
  fontsDir?: string | null;
  forceStyle?: string | null;
};

/** 一次转码的编码配置。缺省是 MP4、AV1、第一条音轨。 */
export type VideoEncode = {
  container?: VideoContainer;
  video?: VideoParams;
  /** 只对转码和裁剪生效。空则保留源帧率。 */
  fps?: number | null;
  size?: VideoSize | null;
  audio?: AudioSelection;
  faststart?: boolean;
  title?: TitleOverlay | null;
  subtitle?: SubtitleOverlay | null;
};

/** 一条音轨。index 是流序号。 */
export type AudioTrack = {
  index: number;
  codec: string;
  sampleRate: number;
  channels: number;
  channelLayout: string;
  bitrateKbps: number;
  language: string;
  title: string;
};

/** 读取到的视频信息。 */
export type VideoInfo = {
  path: string;
  width: number;
  height: number;
  fps: number;
  duration: number;
  videoCodec: string;
  pixFmt: string;
  bitrateKbps: number;
  audioTracks: AudioTrack[];
};

export type JxlColorEncoding = 'srgb' | 'linearSrgb' | 'srgbLuma' | 'linearSrgbLuma';

export type AvifDepth = 'eight' | 'ten' | 'twelve';

export type AvifYuv = 'auto' | '444' | '422' | '420' | '400';

export type AvifRange = 'auto' | 'full' | 'limited';

/** 缺省字段由后端补上：质量 60、速度 6、8-bit、自动分块。 */
export type AvifEncodeParams = {
  qcolor?: number;
  qalpha?: number;
  speed?: number;
  lossless?: boolean;
  jobs?: number | null;
  depth?: AvifDepth;
  yuv?: AvifYuv;
  premultiply?: boolean;
  sharpyuv?: boolean;
  ignoreExif?: boolean;
  ignoreXmp?: boolean;
  ignoreIcc?: boolean;
  range?: AvifRange;
  cicp?: string;
  autotiling?: boolean;
  tilerowslog2?: number | null;
  tilecolslog2?: number | null;
  codec?: string;
  targetSize?: number | null;
  progressive?: boolean;
  pasp?: string;
  crop?: string;
  irot?: number | null;
  imir?: number | null;
  clli?: string;
  advanced?: string[];
};

/** quality 是距离（0 无损），缺省 1、effort 7。页面没填的项不传，后端保持编码器默认。 */
export type JxlEncodeParams = {
  lossless?: boolean;
  quality?: number;
  effort?: number;
  losslessJpeg?: boolean;
  useContainer?: boolean;
  decodingSpeed?: number;
  colorEncoding?: JxlColorEncoding | null;
  targetIntensity?: number | null;
  alphaDistance?: number | null;
  progressive?: boolean;
  groupOrder?: number | null;
  compressBoxes?: number | null;
  brotliEffort?: number | null;
  modular?: number | null;
  numThreads?: number | null;
  photonNoiseIso?: number | null;
  allowJpegReconstruction?: number | null;
  codestreamLevel?: number | null;
  buffering?: number | null;
  premultiply?: number | null;
  keepInvisible?: number | null;
  progressiveAc?: boolean;
  qprogressiveAc?: boolean;
  progressiveDc?: number | null;
  resampling?: number | null;
  ecResampling?: number | null;
  alreadyDownsampled?: boolean;
  upsamplingMode?: number | null;
  epf?: number | null;
  gaborish?: number | null;
  overrideBitdepth?: number | null;
  noise?: number | null;
  dots?: number | null;
  patches?: number | null;
  extraHints?: string[];
};

/** 入队参数。密码只在加解密这一次传入。 */
export type TaskInput =
  | { kind: 'encryptFile'; input: string; output: string; password: string }
  | { kind: 'decryptFile'; input: string; output: string; password: string }
  | {
      kind: 'importUpload';
      source: string;
      owner: OwnerKind;
      ownerId: string;
      uploadKind: UploadKind;
      encrypt: boolean;
      password: string;
    }
  | { kind: 'encryptUpload'; uploadId: string; password: string }
  | { kind: 'decryptUpload'; uploadId: string; password: string }
  | { kind: 'convertVideo'; input: string; output: string; encode?: VideoEncode }
  | { kind: 'cutVideo'; input: string; output: string; segments: TimeSegment[]; encode?: VideoEncode }
  | { kind: 'mergeVideo'; inputs: string[]; output: string; encode?: VideoEncode }
  | {
      kind: 'appendVideo';
      base: string;
      inputs: string[];
      output: string;
      encode?: VideoEncode;
    }
  | { kind: 'convertAvif'; input: string; output: string; params?: AvifEncodeParams }
  | { kind: 'convertJxl'; input: string; output: string; params?: JxlEncodeParams };

/** 把一条耗时文件任务放进队列 */
export function taskEnqueue(input: TaskInput) {
  return invokeCommand<Task>('task_enqueue', { input });
}

/** 当前内存队列 */
export function taskList() {
  return invokeCommand<Task[]>('task_list');
}

/** 取消队列里的任务 */
export function taskCancel(id: string) {
  return invokeCommand<void>('task_cancel', { id });
}

/** 从队列删掉一条。正在处理的不能删。 */
export function taskRemove(id: string) {
  return invokeCommand<void>('task_remove', { id });
}

/** 在系统文件管理器里打开输出文件所在目录，并选中该文件。 */
export function taskReveal(path: string) {
  return invokeCommand<void>('task_reveal', { path });
}

/** 读取视频时长和分辨率 */
export function ffmpegProbe(path: string) {
  return invokeCommand<VideoInfo>('ffmpeg_probe', { path });
}

/** 监听单条任务的进度 */
export function onTaskUpdate(handler: (task: Task) => void): Promise<UnlistenFn> {
  return listen<Task>('task-update', (event) => {
    handler(event.payload);
  });
}

/** 监听队列变化 */
export function onTaskQueueUpdated(handler: (tasks: Task[]) => void): Promise<UnlistenFn> {
  return listen<Task[]>('queue-updated', (event) => {
    handler(event.payload);
  });
}
