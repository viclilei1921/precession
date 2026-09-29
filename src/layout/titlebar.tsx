import { CopySimpleIcon, MinusIcon, SquareIcon, XIcon } from '@phosphor-icons/react';
import { useEffect, useState } from 'react';
import {
  isMobile,
  windowClose,
  windowIsMaximized,
  windowMinimize,
  windowOnResized,
  windowToggleMaximize
} from '@/bridge';
import styles from './titlebar.module.css';

export function TitleBar() {
  const [visible] = useState(() => !isMobile());
  const [maximized, setMaximized] = useState(false);

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
    <header className={styles.bar} data-tauri-drag-region>
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
          <MinusIcon className={styles.icon} weight="thin" />
        </button>
        <button
          type="button"
          className={styles.button}
          title={maximized ? '还原' : '最大化'}
          aria-label={maximized ? '还原' : '最大化'}
          onClick={() => windowToggleMaximize()}
        >
          {maximized ? (
            <CopySimpleIcon className={styles.icon} weight="thin" />
          ) : (
            <SquareIcon className={styles.icon} weight="thin" />
          )}
        </button>
        <button
          type="button"
          className={`${styles.button} ${styles.close}`}
          title="关闭"
          aria-label="关闭"
          onClick={() => windowClose()}
        >
          <XIcon className={styles.icon} weight="thin" />
        </button>
      </div>
    </header>
  );
}
