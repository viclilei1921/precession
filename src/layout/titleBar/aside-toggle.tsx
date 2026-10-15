import { SidebarSimpleIcon } from '@phosphor-icons/react';
import { useAppStore } from '@/store/app';
import styles from './index.module.css';

/** 标题栏上的侧栏展开、收起。 */
export function TitleBarAsideToggle() {
  const collapsed = useAppStore((state) => state.sidebarCollapsed);
  const toggleSidebar = useAppStore((state) => state.toggleSidebar);
  const label = collapsed ? '展开侧栏' : '收起侧栏';

  return (
    <button
      type="button"
      className={styles.navButton}
      data-collapsed={collapsed ? 'true' : undefined}
      title={label}
      aria-expanded={!collapsed}
      aria-label={label}
      onClick={toggleSidebar}
    >
      <SidebarSimpleIcon className={styles.icon} weight="regular" />
    </button>
  );
}
