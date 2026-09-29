import type { UnlistenFn } from '@tauri-apps/api/event';
import { listen } from '@tauri-apps/api/event';

import { invokeCommand } from './invoke';
import type { MediaKind, OwnerKind } from './media';

/** 任务状态 */
export type TaskStatus = 'pending' | 'processing' | 'completed' | 'failed' | 'canceled';

/** 队列里的一条任务 */
export type TaskKind =
  | 'encryptFile'
  | 'decryptFile'
  | 'importMedia'
  | 'encryptMedia'
  | 'decryptMedia'
  | 'convertVideo'
  | 'cutVideo'
  | 'mergeVideo'
  | 'appendVideo'
  | 'convertAvif'
  | 'convertJxl';

/** 队列里的一条任务 */
export type Task = {
  id: string;
  kind: TaskKind;
  status: TaskStatus;
  progress: number;
  message: string;
  mediaId: string | null;
};

/** 裁剪片段。start / duration 为 `HH:MM:SS` 或秒数。 */
export type TimeSegment = {
  start: string;
  duration: string;
};

export type JxlColorEncoding = 'srgb' | 'linearSrgb' | 'srgbLuma' | 'linearSrgbLuma';

/** 缺省字段由后端补上：质量 60、速度 6、自动分块。 */
export type AvifEncodeParams = {
  qcolor?: number;
  qalpha?: number;
  speed?: number;
  lossless?: boolean;
  jobs?: number | null;
  depth?: string;
  yuv?: string;
  premultiply?: boolean;
  sharpyuv?: boolean;
  ignoreExif?: boolean;
  ignoreXmp?: boolean;
  ignoreIcc?: boolean;
  range?: string;
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

/** quality 是距离（0 无损），缺省 1、effort 7。 */
export type JxlEncodeParams = {
  lossless?: boolean;
  quality?: number;
  effort?: number;
  losslessJpeg?: boolean;
  useContainer?: boolean;
  usesOriginalProfile?: boolean;
  decodingSpeed?: number;
  colorEncoding?: JxlColorEncoding | null;
  targetIntensity?: number | null;
  alphaDistance?: number | null;
  progressive?: boolean;
  extraHints?: string[];
};

/** 入队参数。密码只在加解密这一次传入。 */
export type TaskInput =
  | { kind: 'encryptFile'; input: string; output: string; password: string }
  | { kind: 'decryptFile'; input: string; output: string; password: string }
  | {
      kind: 'importMedia';
      source: string;
      owner: OwnerKind;
      ownerId: string;
      mediaKind: MediaKind;
      encrypt: boolean;
      password: string;
    }
  | { kind: 'encryptMedia'; mediaId: string; password: string }
  | { kind: 'decryptMedia'; mediaId: string; password: string }
  | { kind: 'convertVideo'; input: string; output: string; targetFps?: number | null }
  | { kind: 'cutVideo'; input: string; output: string; segments: TimeSegment[] }
  | { kind: 'mergeVideo'; inputs: string[]; output: string; drawFilename?: boolean }
  | { kind: 'appendVideo'; base: string; inputs: string[]; output: string; drawFilename?: boolean }
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
