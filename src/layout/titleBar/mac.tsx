import clsx from 'clsx';
import { Icon } from '@/components/icon';
import styles from './index.module.css';
import { TitleBarNav } from './nav';
import { TitleBarSearch } from './search';

/** mac 系统标题栏 Props */
type TitleBarMacProps = {
  /** 是否全屏 */
  fullscreen: boolean;
};

/** mac 系统标题栏 */
export function TitleBarMac({ fullscreen }: TitleBarMacProps) {
  return (
    <header
      className={clsx(styles.titleBar, styles.macos)}
      data-fullscreen={fullscreen ? 'true' : undefined}
      data-tauri-drag-region
    >
      <div className={styles.leading} data-tauri-drag-region>
        <Icon className={styles.brandIcon} data-tauri-drag-region />
        <TitleBarNav />
      </div>
      <div className={styles.traffic} data-tauri-drag-region="false" />
      <div className={styles.toolbar} data-tauri-drag-region>
        <TitleBarSearch />
      </div>
    </header>
  );
}
