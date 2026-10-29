const STORAGE_KEY = 'precession-appearance';

export type ThemePreference = 'light' | 'dark' | 'system';

export type Appearance = {
  theme: ThemePreference;
  sidebarCollapsed: boolean;
  reduceMotion: boolean;
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
    reduceMotion: systemReduce()
  };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return fallback;
    }
    const parsed = JSON.parse(raw) as Partial<Appearance>;
    return {
      theme: THEMES.includes(parsed.theme as ThemePreference) ? (parsed.theme as ThemePreference) : fallback.theme,
      sidebarCollapsed:
        typeof parsed.sidebarCollapsed === 'boolean' ? parsed.sidebarCollapsed : fallback.sidebarCollapsed,
      reduceMotion: typeof parsed.reduceMotion === 'boolean' ? parsed.reduceMotion : fallback.reduceMotion
    };
  } catch {
    return fallback;
  }
}

export function saveAppearance(value: Appearance) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
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
