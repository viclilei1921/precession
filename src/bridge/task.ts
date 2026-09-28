import type { UnlistenFn } from '@tauri-apps/api/event';
import { listen } from '@tauri-apps/api/event';

import { invokeCommand } from './invoke';
import type { MediaKind, OwnerKind } from './media';

/** 任务状态 */
export type TaskStatus = 'pending' | 'processing' | 'completed' | 'failed' | 'canceled';

/** 队列里的一条任务 */
export type Task = {
  id: string;
  kind: 'encryptFile' | 'decryptFile' | 'importMedia' | 'encryptMedia' | 'decryptMedia';
  status: TaskStatus;
  progress: number;
  message: string;
  mediaId: string | null;
};

/** 入队参数。密码只在这一次传入。 */
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
  | { kind: 'decryptMedia'; mediaId: string; password: string };

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
