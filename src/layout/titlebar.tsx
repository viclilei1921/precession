import { CopySimpleIcon, MagnifyingGlassIcon, MinusIcon, PlusIcon, SquareIcon, XIcon } from '@phosphor-icons/react';
import clsx from 'clsx';
import { useEffect, useRef, useState } from 'react';
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
import { Icon } from '@/components/icon';
import { useTitleBarActions } from '@/store/titlebar';
import styles from './titlebar.module.css';

export function TitleBar() {
  const active = useTitleBarActions((state) => state.active);
  const openCapture = useTitleBarActions((state) => state.openCapture);
  const query = useTitleBarActions((state) => state.query);
  const placeholder = useTitleBarActions((state) => state.placeholder);
  const setQuery = useTitleBarActions((state) => state.setQuery);
  const focusTick = useTitleBarActions((state) => state.focusTick);
  const searchRef = useRef<HTMLInputElement>(null);
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

  useEffect(() => {
    if (focusTick > 0) {
      searchRef.current?.focus();
    }
  }, [focusTick]);

  if (!visible) {
    return <header className={styles.titleBarMobile}></header>;
  }

  return (
    <header
      className={clsx(styles.titleBar, {
        [styles.macos]: macos,
        [styles.fullscreen]: fullscreen,
        [styles.desktop]: active
      })}
      data-tauri-drag-region
    >
      <div className={styles.brand} data-tauri-drag-region>
        <Icon className={styles.brandIcon} />
        <span className={styles.title} data-tauri-drag-region>
          Precession
        </span>
      </div>
      {macos ? <div className={styles.traffic} data-tauri-drag-region="false" /> : null}
      <div className={styles.toolbar} data-tauri-drag-region>
        {active ? (
          <>
            {placeholder ? (
              <label className={styles.search}>
                <MagnifyingGlassIcon className={styles.icon} weight="regular" />
                <input
                  ref={searchRef}
                  className={styles.searchInput}
                  value={query}
                  placeholder={placeholder}
                  aria-label={placeholder}
                  onChange={(event) => setQuery(event.currentTarget.value)}
                />
              </label>
            ) : null}
            <button type="button" className={styles.capture} onClick={openCapture}>
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
