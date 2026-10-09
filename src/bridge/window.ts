import { getCurrentWindow } from '@tauri-apps/api/window';

/** 最小化窗口 */
export function windowMinimize() {
  return getCurrentWindow().minimize();
}

/** 最大化窗口 */
export function windowToggleMaximize() {
  return getCurrentWindow().toggleMaximize();
}

/** 关闭窗口 */
export function windowClose() {
  return getCurrentWindow().close();
}

/** 是否最大化 */
export function windowIsMaximized() {
  return getCurrentWindow().isMaximized();
}

/** 是否全屏 */
export function windowIsFullscreen() {
  return getCurrentWindow().isFullscreen();
}

/** 窗口大小改变时触发 */
export function windowOnResized(handler: () => void) {
  return getCurrentWindow().onResized(handler);
}
