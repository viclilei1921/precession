import { CopySimpleIcon, MinusIcon, SquareIcon, XIcon } from '@phosphor-icons/react';
import clsx from 'clsx';
import { windowClose, windowMinimize, windowToggleMaximize } from '@/bridge';
import { Icon } from '@/components/icon';
import styles from './index.module.css';
import { TitleBarNav } from './nav';
import { TitleBarSearch } from './search';

/** windows 系统标题栏 Props */
type TitleBarWindowsProps = {
  /** 是否最大化 */
  maximized: boolean;
};

/** windows 系统标题栏 */
export function TitleBarWindows({ maximized }: TitleBarWindowsProps) {
  return (
    <header className={styles.titleBar} data-tauri-drag-region>
      <div className={styles.leading}>
        <Icon className={styles.brandIcon} />
        <TitleBarNav />
      </div>
      <div className={styles.toolbar}>
        <TitleBarSearch />
      </div>
      <div className={styles.controls}>
        <button
          type="button"
          className={styles.navButton}
          title="最小化"
          aria-label="最小化"
          onClick={() => windowMinimize()}
        >
          <MinusIcon className={styles.icon} weight="regular" />
        </button>
        <button
          type="button"
          className={styles.navButton}
          title={maximized ? '还原' : '最大化'}
          aria-label={maximized ? '还原' : '最大化'}
          onClick={() => windowToggleMaximize()}
        >
          {maximized ? <CopySimpleIcon className={styles.icon} /> : <SquareIcon className={styles.icon} />}
        </button>
        <button
          type="button"
          className={clsx(styles.navButton, styles.close)}
          title="关闭"
          aria-label="关闭"
          onClick={() => windowClose()}
        >
          <XIcon className={styles.icon} weight="regular" />
        </button>
      </div>
    </header>
  );
}
