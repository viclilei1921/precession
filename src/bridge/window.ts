import { getCurrentWindow } from '@tauri-apps/api/window';

export function windowMinimize() {
  return getCurrentWindow().minimize();
}

export function windowToggleMaximize() {
  return getCurrentWindow().toggleMaximize();
}

export function windowClose() {
  return getCurrentWindow().close();
}

export function windowIsMaximized() {
  return getCurrentWindow().isMaximized();
}

export function windowOnResized(handler: () => void) {
  return getCurrentWindow().onResized(handler);
}
