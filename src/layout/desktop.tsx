import { Link, Outlet } from '@tanstack/react-router';
import { LockButton } from '@/layout/layer/lock-button';
import { itemsOn, sidebarGroups } from '@/router/items';
import styles from './layout.module.css';

export function DesktopLayout() {
  return (
    <div className={styles.desktop}>
      <aside className={styles.nav}>
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
                  activeOptions={{ exact: false }}
                >
                  <item.icon className={styles.icon} weight="regular" />
                  {item.label}
                </Link>
              ))}
          </nav>
        ))}
        <LockButton />
      </aside>
      <div className={styles.main}>
        <div className={styles.content}>
          <Outlet />
        </div>
      </div>
    </div>
  );
}
