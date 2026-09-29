import { platform } from '@tauri-apps/plugin-os';

function isMacos() {
  return platform() === 'macos';
}

function isMobile() {
  const currentPlatform = platform();
  return currentPlatform === 'android' || currentPlatform === 'ios';
}

export { isMacos, isMobile, platform };
