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

export default function App() {
  const unlockedDb = useAppStore((state) => state.dbStatus?.unlocked);

  return (
    <div className={styles.app}>
      <SystemTheme />
      <TitleBar />
      {unlockedDb ? <ContentView /> : <LockView />}
    </div>
  );
}
