import { convertFileSrc } from '@tauri-apps/api/core';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { listen } from '@tauri-apps/api/event';

import { invokeCommand } from './invoke';

/** 查看器接受的扩展名，不带点 */
export const VIEWER_EXTENSIONS = ['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'avif', 'jxl'] as const;

/** 同一层里的一张图片 */
export type ViewerFile = {
  path: string;
  name: string;
  extension: string;
  bytes: number;
  modifiedMs: number;
  width: number | null;
  height: number | null;
};

/** 一个文件夹里可查看的图片 */
export type ViewerFolder = {
  directory: string;
  files: ViewerFile[];
  index: number;
};

/** 可以交给页面显示的文件。JPEG XL 的路径是缓存 PNG */
export type ViewerDisplay = {
  displayPath: string;
  width: number;
  height: number;
};

/** 系统用图片打开本程序时发出的事件 */
export const VIEWER_OPEN_EVENT = 'viewer-open';

/** 列出图片所在文件夹这一层 */
export function viewerFolder(path: string) {
  return invokeCommand<ViewerFolder>('viewer_folder', { path });
}

/** 准备一张可以显示的图。JPEG XL 会先解码 */
export function viewerDisplay(path: string) {
  return invokeCommand<ViewerDisplay>('viewer_display', { path });
}

/** 取走启动时暂存的图片路径。没有时为空 */
export function viewerTakePending() {
  return invokeCommand<string | null>('viewer_take_pending');
}

/** 登记为这些图片的默认打开方式 */
export function viewerRegisterDefault() {
  return invokeCommand<void>('viewer_register_default');
}

/** 监听系统再次打开图片。调用方在卸载时执行返回的函数 */
export function onViewerOpen(handler: (path: string) => void): Promise<UnlistenFn> {
  return listen<string>(VIEWER_OPEN_EVENT, (event) => {
    if (event.payload) {
      handler(event.payload);
    }
  });
}

/** 浏览器可以直接显示的图片地址。JPEG XL 要先解码，这里不处理 */
export function viewerDirectSrc(path: string) {
  return convertFileSrc(path);
}

const displayCache = new Map<string, Promise<string>>();

/** 取一张图的显示地址。同一路径只准备一次 */
export function viewerSrc(file: ViewerFile): Promise<string> {
  const cached = displayCache.get(file.path);
  if (cached) {
    return cached;
  }
  const pending =
    file.extension === 'jxl'
      ? viewerDisplay(file.path).then((display) => convertFileSrc(display.displayPath))
      : Promise.resolve(convertFileSrc(file.path));
  displayCache.set(file.path, pending);
  pending.catch(() => {
    displayCache.delete(file.path);
  });
  return pending;
}
