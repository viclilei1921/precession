import { useAppStore } from '@/store/app';
import { TitleBarAsideToggle } from './aside-toggle';
import { TitleBarHistory } from './history';
import styles from './index.module.css';

/** 解锁后才显示的侧栏开关和路由历史。 */
export function TitleBarNav() {
  const unlocked = useAppStore((state) => Boolean(state.dbStatus?.unlocked));

  if (!unlocked) {
    return null;
  }

  return (
    <div className={styles.navButtons}>
      <TitleBarAsideToggle />
      <TitleBarHistory />
    </div>
  );
}
