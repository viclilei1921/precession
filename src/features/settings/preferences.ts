import { readPreference, writePreference } from '@/bridge/preferences';

const PREFERENCE_KEY = 'appearance';

export type ThemePreference = 'light' | 'dark' | 'system';

export type Appearance = {
  theme: ThemePreference;
  sidebarCollapsed: boolean;
  reduceMotion: boolean;
  /** 导入 md 图片时是否先转成 AVIF。只影响桌面端，移动端始终不转 */
  imageConvert: boolean;
};

const THEMES: ThemePreference[] = ['light', 'dark', 'system'];

function systemDark() {
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}

function systemReduce() {
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

export function loadAppearance(): Appearance {
  const fallback: Appearance = {
    theme: 'light',
    sidebarCollapsed: false,
    reduceMotion: systemReduce(),
    imageConvert: true
  };
  const parsed = readPreference<Partial<Appearance>>(PREFERENCE_KEY);
  if (!parsed) {
    return fallback;
  }
  return {
    theme: THEMES.includes(parsed.theme as ThemePreference) ? (parsed.theme as ThemePreference) : fallback.theme,
    sidebarCollapsed:
      typeof parsed.sidebarCollapsed === 'boolean' ? parsed.sidebarCollapsed : fallback.sidebarCollapsed,
    reduceMotion: typeof parsed.reduceMotion === 'boolean' ? parsed.reduceMotion : fallback.reduceMotion,
    imageConvert: typeof parsed.imageConvert === 'boolean' ? parsed.imageConvert : fallback.imageConvert
  };
}

export function saveAppearance(value: Appearance) {
  void writePreference(PREFERENCE_KEY, value);
}

export function resolveTheme(preference: ThemePreference): 'light' | 'dark' {
  if (preference === 'system') {
    return systemDark() ? 'dark' : 'light';
  }
  return preference;
}

/** 把主题和动效写到文档根节点。侧栏状态由 store 自己驱动。 */
export function applyAppearance(value: Appearance) {
  const root = document.documentElement;
  root.classList.toggle('dark', resolveTheme(value.theme) === 'dark');
  if (value.reduceMotion) {
    root.dataset.motion = 'reduced';
  } else {
    delete root.dataset.motion;
  }
}
