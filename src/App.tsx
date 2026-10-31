import { useEffect } from 'react';
import { TitleBar } from '@/layout/titleBar';
import { useAppStore } from '@/store/app';
import styles from './app.module.css';
import { ContentView } from './layout/content';
import { LockView } from './layout/lock';

function SystemTheme() {
  const preference = useAppStore((state) => state.themePreference);
  const syncSystemTheme = useAppStore((state) => state.syncSystemTheme);

  useEffect(() => {
    if (preference !== 'system') {
      return;
    }
    const media = window.matchMedia('(prefers-color-scheme: dark)');
    const onChange = () => syncSystemTheme();
    media.addEventListener('change', onChange);
    return () => media.removeEventListener('change', onChange);
  }, [preference, syncSystemTheme]);

  return null;
}

/** 移动端打开已经写好的触摸和单列样式。平台在启动时就定了。 */
function MobilePlatform() {
  const mobile = useAppStore((state) => state.isMobile());

  useEffect(() => {
    const root = document.documentElement;
    if (!mobile) {
      delete root.dataset.platform;
      delete root.dataset.touchUi;
      return;
    }
    root.dataset.platform = 'mobile';
    root.dataset.touchUi = 'true';
  }, [mobile]);

  return null;
}

export default function App() {
  const unlockedDb = useAppStore((state) => state.dbStatus?.unlocked);

  return (
    <div className={styles.app}>
      <SystemTheme />
      <MobilePlatform />
      <TitleBar />
      {unlockedDb ? <ContentView /> : <LockView />}
    </div>
  );
}
