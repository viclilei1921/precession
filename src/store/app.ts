import { create } from 'zustand';
import type { DbStatus, Platform } from '@/bridge';
import { platform } from '@/bridge';

type AppState = {
  /** 主题 */
  theme: 'light' | 'dark';
  /** 设备平台 */
  platform: Platform;
  /** 数据库状态 */
  dbStatus: DbStatus | null;
};

type AppActions = {
  /** 启动后设备类型不会改变, 所以这里直接返回是否为移动端(Android 或 iOS) */
  isMobile: () => boolean;
  /** 设置主题 */
  setTheme: (theme: 'light' | 'dark') => void;
  /** 设置数据库状态 */
  setDbStatus: (dbStatus: DbStatus | null) => void;
};

/** 应用状态 */
export const useAppStore = create<AppState & AppActions>((set, get) => ({
  theme: 'light',
  platform: platform(),
  dbStatus: null,
  isMobile: () => {
    const current = get().platform;
    return current === 'android' || current === 'ios';
  },
  setTheme: (theme) => set({ theme }),
  setDbStatus: (dbStatus) => set({ dbStatus })
}));
