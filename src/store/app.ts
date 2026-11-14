import { create } from 'zustand';
import type { DbStatus, Platform } from '@/bridge';
import { platform } from '@/bridge';
import type { Appearance, ThemePreference } from '@/features/settings/preferences';
import { applyAppearance, loadAppearance, resolveTheme, saveAppearance } from '@/features/settings/preferences';

type AppState = {
  /** 用户选择的主题。`system` 表示跟随系统 */
  themePreference: ThemePreference;
  /** 实际生效的主题 */
  theme: 'light' | 'dark';
  /** 关闭过渡和渐隐 */
  reduceMotion: boolean;
  /** 导入 md 图片时是否先转 AVIF（桌面端生效） */
  imageConvert: boolean;
  /** 设备平台 */
  platform: Platform;
  /** 数据库状态 */
  dbStatus: DbStatus | null;
  /** 桌面侧栏是否收起 */
  sidebarCollapsed: boolean;
};

type AppActions = {
  /** 启动后设备类型不会改变, 所以这里直接返回是否为移动端(Android 或 iOS) */
  isMobile: () => boolean;
  /** 设置主题偏好，并立刻应用到文档 */
  setThemePreference: (theme: ThemePreference) => void;
  /** 系统外观变化时，只在「跟随系统」下更新实际主题 */
  syncSystemTheme: () => void;
  /** 设置是否减少动效 */
  setReduceMotion: (reduceMotion: boolean) => void;
  /** 设置导入图片是否自动转码 */
  setImageConvert: (imageConvert: boolean) => void;
  /** 设置数据库状态 */
  setDbStatus: (dbStatus: DbStatus | null) => void;
  /** 直接设置侧栏收起状态 */
  setSidebarCollapsed: (sidebarCollapsed: boolean) => void;
  /** 切换桌面侧栏展开和收起 */
  toggleSidebar: () => void;
};

function persist(get: () => AppState, patch: Partial<Appearance>) {
  const current = get();
  const next: Appearance = {
    theme: patch.theme ?? current.themePreference,
    sidebarCollapsed: patch.sidebarCollapsed ?? current.sidebarCollapsed,
    reduceMotion: patch.reduceMotion ?? current.reduceMotion,
    imageConvert: patch.imageConvert ?? current.imageConvert
  };
  saveAppearance(next);
  applyAppearance(next);
  return next;
}

const loaded = loadAppearance();

/** 应用状态 */
export const useAppStore = create<AppState & AppActions>((set, get) => ({
  themePreference: loaded.theme,
  theme: resolveTheme(loaded.theme),
  reduceMotion: loaded.reduceMotion,
  imageConvert: loaded.imageConvert,
  platform: platform(),
  dbStatus: null,
  sidebarCollapsed: loaded.sidebarCollapsed,
  isMobile: () => {
    const current = get().platform;
    return current === 'android' || current === 'ios';
  },
  setThemePreference: (themePreference) => {
    persist(get, { theme: themePreference });
    set({ themePreference, theme: resolveTheme(themePreference) });
  },
  syncSystemTheme: () => {
    if (get().themePreference !== 'system') {
      return;
    }
    applyAppearance({
      theme: 'system',
      sidebarCollapsed: get().sidebarCollapsed,
      reduceMotion: get().reduceMotion,
      imageConvert: get().imageConvert
    });
    set({ theme: resolveTheme('system') });
  },
  setReduceMotion: (reduceMotion) => {
    persist(get, { reduceMotion });
    set({ reduceMotion });
  },
  setImageConvert: (imageConvert) => {
    persist(get, { imageConvert });
    set({ imageConvert });
  },
  setDbStatus: (dbStatus) => set({ dbStatus }),
  setSidebarCollapsed: (sidebarCollapsed) => {
    persist(get, { sidebarCollapsed });
    set({ sidebarCollapsed });
  },
  toggleSidebar: () => get().setSidebarCollapsed(!get().sidebarCollapsed)
}));

/**
 * 导入 md 图片时是否先转成 AVIF。
 * 桌面端跟随「外观」里的开关，移动端始终不转（转码太慢）。
 */
export function convertImagesOnImport() {
  const state = useAppStore.getState();
  return state.imageConvert && !state.isMobile();
}

/**
 * 用 Store 里读到的外观偏好刷新状态。
 *
 * 模块加载时 Store 还没预载，`loadAppearance` 会先用默认值。启动流程在
 * `preloadPreferences` 之后调用这里，避免首屏闪主题。
 */
export function hydrateAppearance() {
  const appearance = loadAppearance();
  applyAppearance(appearance);
  useAppStore.setState({
    themePreference: appearance.theme,
    theme: resolveTheme(appearance.theme),
    reduceMotion: appearance.reduceMotion,
    imageConvert: appearance.imageConvert,
    sidebarCollapsed: appearance.sidebarCollapsed
  });
}
