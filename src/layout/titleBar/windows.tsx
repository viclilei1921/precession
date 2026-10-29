import { CopySimpleIcon, MinusIcon, SquareIcon, XIcon } from '@phosphor-icons/react';
import { windowClose, windowMinimize, windowToggleMaximize } from '@/bridge';
import { IconButton } from '@/components/button/icon';
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
        <IconButton label="最小化" size="compact" onClick={() => windowMinimize()}>
          <MinusIcon weight="regular" />
        </IconButton>
        <IconButton label={maximized ? '还原' : '最大化'} size="compact" onClick={() => windowToggleMaximize()}>
          {maximized ? <CopySimpleIcon weight="regular" /> : <SquareIcon weight="regular" />}
        </IconButton>
        <IconButton label="关闭" size="compact" tone="destructive" onClick={() => windowClose()}>
          <XIcon weight="regular" />
        </IconButton>
      </div>
    </header>
  );
}
