import type { UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { open } from '@tauri-apps/plugin-dialog';

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
  return Array.isArray(selected) ? selected : [selected];
}
