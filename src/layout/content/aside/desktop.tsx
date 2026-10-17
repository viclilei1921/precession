import { CaretDownIcon } from '@phosphor-icons/react';
import { Link, useRouterState } from '@tanstack/react-router';
import clsx from 'clsx';
import { useEffect, useState } from 'react';
import { rippleHost, useRipple } from '@/components/ripple';
import { Scroll } from '@/components/scroll';
import type { RouteGroup, RouteItem, RouteTone } from '@/router/items';
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

function covers(pathname: string, item: RouteItem) {
  return pathname === item.to || pathname.startsWith(`${item.to}/`);
}

function MoreButton({ open, onClick }: { open: boolean; onClick: () => void }) {
  const ripple = useRipple();

  return (
    <button
      type="button"
      className={clsx(rippleHost, styles.link, styles.more)}
      aria-expanded={open}
      data-open={open ? 'true' : undefined}
      onClick={onClick}
      onPointerDown={ripple.onPointerDown}
      onKeyDown={ripple.onKeyDown}
    >
      {ripple.nodes}
      <CaretDownIcon className={styles.icon} weight="regular" />
      <span className={styles.label}>{open ? '收起' : '更多'}</span>
    </button>
  );
}

function AsideGroup({ group }: { group: RouteGroup }) {
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const items = itemsOn('sidebar').filter((item) => item.group === group);
  const primary = items.filter((item) => item.fold !== 'more');
  const more = items.filter((item) => item.fold === 'more');
  const foldedHere = more.some((item) => covers(pathname, item));
  const [open, setOpen] = useState(foldedHere);

  useEffect(() => {
    const onFolded = itemsOn('sidebar').some(
      (item) => item.group === group && item.fold === 'more' && covers(pathname, item)
    );
    if (onFolded) {
      setOpen(true);
    }
  }, [group, pathname]);

  return (
    <nav className={styles.group} aria-label={group}>
      <div className={styles.groupLabel}>{group}</div>
      {primary.map((item) => (
        <AsideLink key={item.id} item={item} />
      ))}
      {more.length > 0 ? (
        <div className={styles.moreBlock}>
          <div className={styles.fold} data-open={open ? 'true' : undefined} inert={open ? undefined : true}>
            <div className={styles.foldInner}>
              {more.map((item) => (
                <AsideLink key={item.id} item={item} />
              ))}
            </div>
          </div>
          <MoreButton open={open} onClick={() => setOpen((value) => !value)} />
        </div>
      ) : null}
    </nav>
  );
}

/** 桌面侧栏。展开和收起由标题栏控制，纸面整页滑入滑出。 */
export function DesktopAside() {
  const collapsed = useAppStore((state) => state.sidebarCollapsed);

  return (
    <aside className={styles.nav} data-collapsed={collapsed ? 'true' : undefined}>
      <Scroll className={styles.paper}>
        <div className={styles.stack}>
          {sidebarGroups.map((group) => (
            <AsideGroup key={group} group={group} />
          ))}
        </div>
      </Scroll>
    </aside>
  );
}
