import { Link } from '@tanstack/react-router';
import clsx from 'clsx';
import { rippleHost, useRipple } from '@/components/ripple';
import type { RouteItem, RouteTone } from '@/router/items';
import { itemsOn, sidebarGroups } from '@/router/items';
import { useAppStore } from '@/store/app';
import styles from './index.module.css';

const toneClass: Record<RouteTone, string> = {
  plan: styles.tonePlan,
  journal: styles.toneJournal,
  growth: styles.toneGrowth,
  library: styles.toneLibrary,
  toolbox: styles.toneToolbox
};

function AsideLink({ item }: { item: RouteItem }) {
  const ripple = useRipple();

  return (
    <Link
      to={item.to}
      className={clsx(rippleHost, styles.link, item.tone && toneClass[item.tone])}
      aria-label={item.label}
      title={item.label}
      activeOptions={{ exact: false }}
      onPointerDown={ripple.onPointerDown}
      onKeyDown={ripple.onKeyDown}
    >
      {ripple.nodes}
      <item.icon className={styles.icon} weight="regular" />
      <span className={styles.label}>{item.label}</span>
    </Link>
  );
}

/** 桌面侧栏。展开和收起由标题栏控制，纸面整页滑入滑出。 */
export function DesktopAside() {
  const collapsed = useAppStore((state) => state.sidebarCollapsed);

  return (
    <aside className={styles.nav} data-collapsed={collapsed ? 'true' : undefined}>
      <div className={styles.paper}>
        {sidebarGroups.map((group) => (
          <nav key={group} className={styles.group} aria-label={group}>
            <div className={styles.groupLabel}>{group}</div>
            {itemsOn('sidebar')
              .filter((item) => item.group === group)
              .map((item) => (
                <AsideLink key={item.id} item={item} />
              ))}
          </nav>
        ))}
      </div>
    </aside>
  );
}
