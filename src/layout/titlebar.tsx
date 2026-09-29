import { CopySimpleIcon, MinusIcon, SquareIcon, XIcon } from '@phosphor-icons/react';
import { useEffect, useState } from 'react';
import { windowClose, windowIsMaximized, windowMinimize, windowOnResized, windowToggleMaximize } from '@/bridge';
import { isMobilePlatform } from '@/platform';
import styles from './titlebar.module.css';

function isTauri() {
  return '__TAURI_INTERNALS__' in window;
}

export function TitleBar() {
  const [visible, setVisible] = useState(() => isTauri() && !isMobilePlatform());
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    const root = document.documentElement;
    const sync = () => setVisible(isTauri() && root.dataset.platform !== 'mobile');
    const observer = new MutationObserver(sync);
    observer.observe(root, { attributes: true, attributeFilter: ['data-platform'] });
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    if (!visible) {
      return;
    }

    let unlisten: (() => void) | undefined;
    let cancelled = false;

    windowIsMaximized().then((value) => {
      if (!cancelled) {
        setMaximized(value);
      }
    });

    windowOnResized(() => {
      windowIsMaximized().then((value) => {
        if (!cancelled) {
          setMaximized(value);
        }
      });
    }).then((stop) => {
      if (cancelled) {
        stop();
        return;
      }
      unlisten = stop;
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [visible]);

  if (!visible) {
    return null;
  }

  return (
    <header className={styles.bar}>
      <div className={styles.drag} data-tauri-drag-region>
        <span className={styles.title} data-tauri-drag-region>
          Precession
        </span>
      </div>
      <div className={styles.controls}>
        <button
          type="button"
          className={styles.button}
          title="最小化"
          aria-label="最小化"
          onClick={() => windowMinimize()}
        >
          <MinusIcon className={styles.icon} weight="bold" />
        </button>
        <button
          type="button"
          className={styles.button}
          title={maximized ? '还原' : '最大化'}
          aria-label={maximized ? '还原' : '最大化'}
          onClick={() => windowToggleMaximize()}
        >
          {maximized ? (
            <CopySimpleIcon className={styles.icon} weight="bold" />
          ) : (
            <SquareIcon className={styles.icon} weight="regular" />
          )}
        </button>
        <button
          type="button"
          className={`${styles.button} ${styles.close}`}
          title="关闭"
          aria-label="关闭"
          onClick={() => windowClose()}
        >
          <XIcon className={styles.icon} weight="bold" />
        </button>
      </div>
    </header>
  );
}
