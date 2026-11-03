import { isTauri } from '@tauri-apps/api/core';
import { LazyStore } from '@tauri-apps/plugin-store';

/**
 * 应用偏好的持久化层。
 *
 * Tauri 里使用官方 Store 插件（`preferences.json`），浏览器调试时退回 localStorage。
 * 启动时先 `preloadPreferences` 把值读进内存，之后用同步的 `readPreference` 读取，
 * 避免 React 首屏因异步 IO 闪一下中文。
 */

export const PREFERENCES_FILE = 'preferences.json';

/** 旧版本用的 localStorage 前缀，首次启动时迁移到 store */
const LEGACY_PREFIX = 'precession-';

export type PreferenceKey = 'language' | 'appearance';

const cache = new Map<string, unknown>();

let store: LazyStore | null = null;

function openStore(): LazyStore {
  store ??= new LazyStore(PREFERENCES_FILE, { autoSave: 200 });
  return store;
}

function legacyKey(key: string): string {
  return `${LEGACY_PREFIX}${key}`;
}

function readLegacy(key: string): unknown {
  try {
    const raw = localStorage.getItem(legacyKey(key));
    if (raw == null) {
      return undefined;
    }
    try {
      return JSON.parse(raw) as unknown;
    } catch {
      return raw;
    }
  } catch {
    return undefined;
  }
}

function writeLegacy(key: string, value: unknown): void {
  try {
    localStorage.setItem(legacyKey(key), typeof value === 'string' ? value : JSON.stringify(value));
  } catch {
    /* 浏览器隐私模式下忽略 */
  }
}

/** 启动时预读偏好。必须在渲染 React 之前 await。 */
export async function preloadPreferences(keys: PreferenceKey[]): Promise<void> {
  if (!isTauri()) {
    for (const key of keys) {
      const value = readLegacy(key);
      if (value !== undefined) {
        cache.set(key, value);
      }
    }
    return;
  }

  try {
    const handle = openStore();
    for (const key of keys) {
      const stored = await handle.get<unknown>(key);
      if (stored !== undefined && stored !== null) {
        cache.set(key, stored);
        continue;
      }
      // 旧版本只写在 localStorage，迁移一次。
      const legacy = readLegacy(key);
      if (legacy !== undefined) {
        cache.set(key, legacy);
        await handle.set(key, legacy);
      }
    }
    await handle.save();
  } catch (error) {
    console.warn('[preferences] failed to load store, falling back to localStorage', error);
    for (const key of keys) {
      const value = readLegacy(key);
      if (value !== undefined) {
        cache.set(key, value);
      }
    }
  }
}

/** 同步读取已预载的偏好。未预载时退回旧 localStorage。 */
export function readPreference<T>(key: PreferenceKey): T | null {
  if (cache.has(key)) {
    return cache.get(key) as T;
  }
  const legacy = readLegacy(key);
  return legacy === undefined ? null : (legacy as T);
}

/** 写入偏好。Store 里异步落盘，浏览器里写 localStorage。 */
export async function writePreference(key: PreferenceKey, value: unknown): Promise<void> {
  cache.set(key, value);
  if (!isTauri()) {
    writeLegacy(key, value);
    return;
  }
  try {
    const handle = openStore();
    await handle.set(key, value);
    await handle.save();
  } catch (error) {
    console.warn('[preferences] failed to persist preference, falling back to localStorage', error);
    writeLegacy(key, value);
  }
}
