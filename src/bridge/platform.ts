import { platform } from '@tauri-apps/plugin-os';

function isMobile() {
  const currentPlatform = platform();
  return currentPlatform === 'android' || currentPlatform === 'ios';
}

export { isMobile, platform };
