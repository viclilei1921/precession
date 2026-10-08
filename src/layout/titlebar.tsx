import { CopySimpleIcon, MinusIcon, SquareIcon, XIcon } from '@phosphor-icons/react';
import clsx from 'clsx';
import { useEffect, useState } from 'react';
import {
  isMacos,
  isMobile,
  windowClose,
  windowIsFullscreen,
  windowIsMaximized,
  windowMinimize,
  windowOnResized,
  windowToggleMaximize
} from '@/bridge';
import styles from './titlebar.module.css';

export function TitleBar() {
  // 是否显示标题栏 非移动端显示
  const [visible] = useState(() => !isMobile());
  const [macos] = useState(() => isMacos());
  const [maximized, setMaximized] = useState(false);
  const [fullscreen, setFullscreen] = useState(false);

  useEffect(() => {
    if (!visible || !macos) {
      return;
    }

    let unlisten: (() => void) | undefined;
    let cancelled = false;

    const sync = () => {
      windowIsFullscreen().then((value) => {
        if (!cancelled) {
          setFullscreen(value);
        }
      });
    };

    sync();
    windowOnResized(sync).then((stop) => {
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
  }, [macos, visible]);

  useEffect(() => {
    if (!visible || macos) {
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
  }, [macos, visible]);

  if (!visible) {
    return <header className={styles.titleBarMobile}></header>;
  }

  return (
    <header
      className={clsx(styles.titleBar, {
        [styles.macos]: macos,
        [styles.fullscreen]: fullscreen
      })}
      data-tauri-drag-region
    >
      <div className={styles.drag} data-tauri-drag-region>
        <span className={styles.title} data-tauri-drag-region>
          Precession
        </span>
      </div>
      {macos ? <div className={styles.traffic} data-tauri-drag-region="false" /> : null}
      {macos ? null : (
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
      )}
    </header>
  );
}
