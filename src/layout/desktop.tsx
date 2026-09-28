import { Link, Outlet } from '@tanstack/react-router';
import { itemsOn, sidebarGroups } from '@/router/items';
import { LockButton } from '@/session/lock-button';
import styles from './layout.module.css';

type DesktopLayoutProps = {
  onCapture: () => void;
  onCommand: () => void;
};

export function DesktopLayout({ onCapture, onCommand }: DesktopLayoutProps) {
  return (
    <div className={styles.desktop}>
      <aside className={styles.nav}>
        <p className={styles.brand}>Precession</p>
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
                  {item.label}
                </Link>
              ))}
          </nav>
        ))}
        <LockButton />
      </aside>
      <div className={styles.main}>
        <header className={styles.topbar}>
          <button type="button" className={styles.search} onClick={onCommand}>
            搜索全部记录
          </button>
          <button type="button" className={styles.capture} onClick={onCapture}>
            记一笔
          </button>
        </header>
        <div className={styles.content}>
          <Outlet />
        </div>
      </div>
    </div>
  );
}
