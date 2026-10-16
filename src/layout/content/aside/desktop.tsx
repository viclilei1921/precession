import { Link } from '@tanstack/react-router';
import clsx from 'clsx';
import { itemsOn, sidebarGroups } from '@/router/items';
import { useAppStore } from '@/store/app';
import styles from './index.module.css';

/** 桌面侧栏。展开和收起由标题栏控制。 */
export function DesktopAside() {
  const collapsed = useAppStore((state) => state.sidebarCollapsed);

  return (
    <aside className={clsx(styles.nav, collapsed && styles.collapsed)}>
      {sidebarGroups.map((group) => (
        <nav key={group} className={styles.group} aria-label={group}>
          <div className={styles.groupLabel}>{group}</div>
          {itemsOn('sidebar')
            .filter((item) => item.group === group)
            .map((item) => (
              <Link
                key={item.id}
                to={item.to}
                data-tone={item.tone}
                className={styles.link}
                aria-label={item.label}
                title={item.label}
                activeOptions={{ exact: false }}
              >
                <item.icon className={styles.icon} weight="regular" />
                <span className={styles.label}>{item.label}</span>
              </Link>
            ))}
        </nav>
      ))}
    </aside>
  );
}
