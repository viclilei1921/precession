import clsx from 'clsx';
import { Icon } from '@/components/icon';
import styles from './index.module.css';
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
      className={clsx(styles.titleBar, styles.macos, {
        [styles.fullscreen]: fullscreen
      })}
      data-tauri-drag-region
    >
      <div className={styles.brand}>
        <Icon className={styles.brandIcon} />
        <span className={styles.title}>Precession</span>
      </div>
      <div className={styles.traffic} data-tauri-drag-region="false" />
      <div className={styles.toolbar}>
        <TitleBarSearch />
      </div>
    </header>
  );
}
