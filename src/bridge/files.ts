import type { UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { open } from '@tauri-apps/plugin-dialog';

import { invokeCommand } from './invoke';

/** 窗口拖放所处的阶段 */
export type FileDropPhase = 'enter' | 'over' | 'drop' | 'leave';

export type FileDropPayload = {
  phase: FileDropPhase;
  paths: string[];
};

/** 监听文件拖进当前窗口。调用方在卸载时执行返回的函数。 */
export function onFileDrop(handler: (event: FileDropPayload) => void): Promise<UnlistenFn> {
  return getCurrentWebview().onDragDropEvent((event) => {
    const payload = event.payload;
    if (payload.type === 'enter') {
      handler({ phase: 'enter', paths: payload.paths });
      return;
    }
    if (payload.type === 'drop') {
      handler({ phase: 'drop', paths: payload.paths });
      return;
    }
    if (payload.type === 'over') {
      handler({ phase: 'over', paths: [] });
      return;
    }
    handler({ phase: 'leave', paths: [] });
  });
}

/** 用系统对话框选择文件。取消时返回空数组。 */
export async function pickFiles(options: { multiple?: boolean; extensions?: string[] }) {
  const extensions = options.extensions?.filter((item) => item.length > 0);
  const selected = await open({
    multiple: options.multiple ?? true,
    directory: false,
    fileAccessMode: 'scoped',
    filters: extensions && extensions.length > 0 ? [{ name: '文件', extensions }] : undefined
  });
  if (selected == null) {
    return [];
  }
  const picked = Array.isArray(selected) ? selected : [selected];
  if (!picked.some(needsMaterialize)) {
    return picked;
  }
  return invokeCommand<string[]>('materialize_picked', { paths: picked });
}

/** Android 选择器会返回 content 或 file 地址，要先收成本地文件。 */
function needsMaterialize(path: string) {
  return /^content:/i.test(path) || /^file:/i.test(path);
}

/** 用系统对话框选择一个文件夹。取消时返回空字符串。 */
export async function pickDirectory() {
  const selected = await open({
    multiple: false,
    directory: true,
    fileAccessMode: 'scoped'
  });
  if (typeof selected !== 'string') {
    return '';
  }
  return selected;
}

/** 用系统对话框选择文件夹。取消时返回空数组。 */
export async function pickDirectories() {
  const selected = await open({
    multiple: true,
    directory: true,
    fileAccessMode: 'scoped'
  });
  if (selected == null) {
    return [];
  }
  return Array.isArray(selected) ? selected : [selected];
}

/** 收集文件的结果。`scannedDirectory` 为真表示展开过文件夹。 */
export type CollectedFiles = {
  files: string[];
  scannedDirectory: boolean;
};

/** 展开路径。文件按扩展名保留，文件夹会连同子文件夹里符合的文件一起加入。扩展名为空时保留全部文件。 */
export function collectFiles(paths: string[], extensions: string[]) {
  if (paths.length === 0) {
    return Promise.resolve({ files: [], scannedDirectory: false });
  }
  return invokeCommand<CollectedFiles>('collect_files', { paths, extensions });
}

/** 一个文件的字节大小。读不到的路径不会出现。 */
export type FileSize = {
  path: string;
  size: number;
};

/** 读取文件字节数。目录和读失败的路径会被跳过。 */
export function fileSizes(paths: string[]) {
  if (paths.length === 0) {
    return Promise.resolve([]);
  }
  return invokeCommand<FileSize[]>('file_sizes', { paths });
}
