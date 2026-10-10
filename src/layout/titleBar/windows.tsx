import { CopySimpleIcon, MinusIcon, SquareIcon, XIcon } from '@phosphor-icons/react';
import clsx from 'clsx';
import { windowClose, windowMinimize, windowToggleMaximize } from '@/bridge';
import { Icon } from '@/components/icon';
import { TitleBarCapture } from './capture';
import styles from './index.module.css';
import { TitleBarSearch } from './search';

type TitleBarWindowsProps = {
  maximized: boolean;
};

export function TitleBarWindows({ maximized }: TitleBarWindowsProps) {
  return (
    <header className={styles.titleBar} data-tauri-drag-region>
      <div className={styles.brand} data-tauri-drag-region>
        <Icon className={styles.brandIcon} />
        <span className={styles.title} data-tauri-drag-region>
          Precession
        </span>
      </div>
      <div className={styles.toolbar} data-tauri-drag-region>
        <TitleBarSearch />
        <TitleBarCapture />
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
          className={clsx(styles.button, styles.close)}
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
