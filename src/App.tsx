import { useEffect } from 'react';
import { isTauri } from '@/bridge/platform';
import { onViewerOpen, viewerTakePending } from '@/bridge/viewer';
import { rememberViewerPath } from '@/features/viewer/open';
import { TitleBar } from '@/layout/titleBar';
import { useAppStore } from '@/store/app';
import styles from './app.module.css';
import '@/layout/mobile/back';
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

  useEffect(() => {
    const modified = (event: KeyboardEvent | WheelEvent) => {
      return /mac/i.test(navigator.userAgent) ? event.metaKey : event.ctrlKey;
    };
    const blockBrowserZoom = (event: WheelEvent) => {
      if (modified(event)) {
        event.preventDefault();
      }
    };
    const blockZoomKeys = (event: KeyboardEvent) => {
      if (!modified(event)) {
        return;
      }
      if (event.key === '-' || event.key === '=' || event.key === '+' || event.key === '0') {
        event.preventDefault();
      }
    };
    window.addEventListener('wheel', blockBrowserZoom, { capture: true, passive: false });
    window.addEventListener('keydown', blockZoomKeys, { capture: true });
    return () => {
      window.removeEventListener('wheel', blockBrowserZoom, { capture: true });
      window.removeEventListener('keydown', blockZoomKeys, { capture: true });
    };
  }, []);

  useEffect(() => {
    if (!isTauri()) {
      return;
    }
    let stop: (() => void) | undefined;
    let cancelled = false;
    void (async () => {
      const unlisten = await onViewerOpen((path) => rememberViewerPath(path));
      if (cancelled) {
        unlisten();
        return;
      }
      stop = unlisten;
      const pending = await viewerTakePending();
      if (!cancelled && pending) {
        rememberViewerPath(pending);
      }
    })();
    return () => {
      cancelled = true;
      stop?.();
    };
  }, []);

  return (
    <div className={styles.app}>
      <SystemTheme />
      <MobilePlatform />
      <TitleBar />
      {unlockedDb ? <ContentView /> : <LockView />}
    </div>
  );
}
