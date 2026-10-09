import { platform } from '@tauri-apps/plugin-os';

/** 是否是 macOS 系统 */
function isMacos() {
  return platform() === 'macos';
}

/** 是否是移动端系统 */
function isMobile() {
  const currentPlatform = platform();
  return currentPlatform === 'android' || currentPlatform === 'ios';
}

export { isMacos, isMobile, platform };
