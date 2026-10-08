import { CopySimpleIcon, MagnifyingGlassIcon, MinusIcon, PlusIcon, SquareIcon, XIcon } from '@phosphor-icons/react';
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
import { useTitleBarActions } from './titlebar-actions';

function TitleMark() {
  return (
    <svg className={styles.mark} viewBox="0 0 390 390" aria-hidden="true" data-tauri-drag-region>
      <polygon points="195,0 292.5,97.5 195,195 97.5,97.5" fill="#FFE270" />
      <polygon points="390,195 292.5,292.5 195,195 292.5,97.5" fill="#D8CDE3" />
      <polygon points="195,390 97.5,292.5 195,195 292.5,292.5" fill="#89A9C2" />
      <polygon points="0,195 97.5,97.5 195,195 97.5,292.5" fill="#9CC8B5" />
      <rect x="168" y="168" width="55" height="55" fill="#F8F9FA" />
    </svg>
  );
}

export function TitleBar() {
  const { actions } = useTitleBarActions();
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
        [styles.fullscreen]: fullscreen,
        [styles.desktop]: actions
      })}
      data-tauri-drag-region
    >
      <div className={styles.brand} data-tauri-drag-region>
        <TitleMark />
        <span className={styles.title} data-tauri-drag-region>
          Precession
        </span>
      </div>
      {macos ? <div className={styles.traffic} data-tauri-drag-region="false" /> : null}
      <div className={styles.toolbar} data-tauri-drag-region>
        {actions ? (
          <>
            <button type="button" className={styles.search} onClick={actions.onCommand}>
              <MagnifyingGlassIcon className={styles.icon} weight="regular" />
              <span className={styles.searchLabel}>搜索全部记录</span>
              <span className={styles.kbd}>{macos ? '⌘K' : 'Ctrl K'}</span>
            </button>
            <button type="button" className={styles.capture} onClick={actions.onCapture}>
              <PlusIcon className={styles.icon} weight="regular" />
              记一笔
            </button>
          </>
        ) : null}
      </div>
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
