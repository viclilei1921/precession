import { useEffect, useState } from 'react';
import { windowIsFullscreen, windowIsMaximized, windowOnResized } from '@/bridge';
import { useAppStore } from '@/store/app';
import { debounce } from '@/utils/lodash';
import styles from './index.module.css';
import { TitleBarMac } from './mac';
import { TitleBarWindows } from './windows';

export function TitleBar() {
  /** 是否显示标题栏(移动端不显示) */
  const visible = useAppStore((state) => !state.isMobile());
  /** 是否 macOS 系统 */
  const macos = useAppStore((state) => state.platform === 'macos');
  const window = useAppStore((state) => state.platform === 'windows');

  const [maximized, setMaximized] = useState(false);
  const [fullscreen, setFullscreen] = useState(false);

  useEffect(() => {
    if (!visible) {
      return;
    }

    let unlistenResize: (() => void) | undefined;

    const sync = debounce(async () => {
      const [isMaximized, isFullscreen] = await Promise.all([windowIsMaximized(), windowIsFullscreen()]);

      setMaximized(isMaximized);
      setFullscreen(isFullscreen);
    }, 150);

    const init = async () => {
      setMaximized(await windowIsMaximized());
      setFullscreen(await windowIsFullscreen());

      unlistenResize = await windowOnResized(sync);
    };

    init();

    return () => {
      unlistenResize?.();
    };
  }, [visible]);

  if (!visible) {
    return <header className={styles.titleBarMobile}></header>;
  }

  if (macos) {
    return <TitleBarMac fullscreen={fullscreen} />;
  }

  if (window) {
    return <TitleBarWindows maximized={maximized} />;
  }

  return null;
}
