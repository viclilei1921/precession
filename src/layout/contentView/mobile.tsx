import { BookOpenIcon, PencilSimpleIcon, PlusIcon, SunIcon, UserIcon } from '@phosphor-icons/react';
import { Link, Outlet, useRouterState } from '@tanstack/react-router';
import { LockButton } from '@/layout/layer/lock-button';
import { itemsOn } from '@/router/items';
import { PATH } from '@/router/path';
import styles from './index.module.css';

type MobileContentProps = {
  onCapture: () => void;
};

type TabId = 'today' | 'record' | 'library' | 'mine';

function currentTab(pathname: string): TabId {
  if (pathname.startsWith(PATH.plan) || pathname.startsWith(PATH.journal) || pathname.startsWith(PATH.growth)) {
    return 'record';
  }
  if (pathname.startsWith(PATH.library)) {
    return 'library';
  }
  if (pathname.startsWith(PATH.toolbox) || pathname.startsWith(PATH.review) || pathname.startsWith(PATH.settings)) {
    return 'mine';
  }
  return 'today';
}

/** 移动端内容区：分段导航、当前页面和底栏 */
export function MobileContent({ onCapture }: MobileContentProps) {
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const tab = currentTab(pathname);
  const segments = tab === 'record' ? itemsOn('record') : tab === 'mine' ? itemsOn('mine') : [];

  return (
    <div className={styles.mobile}>
      {segments.length > 0 ? (
        <nav className={styles.segments} aria-label={tab === 'record' ? '记录' : '我的'}>
          {segments.map((item) => (
            <Link key={item.id} to={item.to} className={styles.segment} activeOptions={{ exact: false }}>
              <item.icon className={styles.icon} weight="regular" />
              {item.label}
            </Link>
          ))}
        </nav>
      ) : null}
      <div className={styles.content}>
        <Outlet />
        {tab === 'mine' ? <LockButton /> : null}
      </div>
      <nav className={styles.tabbar} aria-label="主导航">
        <Link to={PATH.today} className={styles.tab} data-status={tab === 'today' ? 'active' : undefined}>
          <SunIcon className={styles.icon} weight="regular" />
          今天
        </Link>
        <Link to={PATH.journal} className={styles.tab} data-status={tab === 'record' ? 'active' : undefined}>
          <PencilSimpleIcon className={styles.icon} weight="regular" />
          记录
        </Link>
        <button type="button" className={styles.fab} aria-label="记一笔" onClick={onCapture}>
          <PlusIcon className={styles.icon} weight="regular" />
        </button>
        <Link to={PATH.library} className={styles.tab} data-status={tab === 'library' ? 'active' : undefined}>
          <BookOpenIcon className={styles.icon} weight="regular" />
          书库
        </Link>
        <Link to={PATH.settings} className={styles.tab} data-status={tab === 'mine' ? 'active' : undefined}>
          <UserIcon className={styles.icon} weight="regular" />
          我的
        </Link>
      </nav>
    </div>
  );
}
