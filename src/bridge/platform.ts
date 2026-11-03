import { isTauri } from '@tauri-apps/api/core';
import type { Platform } from '@tauri-apps/plugin-os';
import { arch, locale, platform, version } from '@tauri-apps/plugin-os';

export { arch, isTauri, locale, type Platform, platform, version };
