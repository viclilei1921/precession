import { QueryClientProvider } from '@tanstack/react-query';
import { Outlet, RouterProvider, useRouterState } from '@tanstack/react-router';
import { TabBar } from '@/layout/mobile/tab-bar';
import { Toast } from '@/layout/mobile/toast';
import { queryClient } from '@/query/client';
import { router } from '@/router';
import { useAppStore } from '@/store/app';
import { DesktopAside } from './aside/desktop';
import styles from './index.module.css';

const MAIN_TABS = new Set(['/plan', '/journal', '/growth', '/library', '/settings', '/files']);

/** 解锁后的内容区。数据和路由从这里开始。 */
export function ContentView() {
  return (
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}

/** 路由根布局：侧栏和功能区出口。 */
export function ContentLayout() {
  const mobile = useAppStore((state) => state.isMobile());
  const pathname = useRouterState({ select: (state) => state.location.pathname });

  if (mobile) {
    return (
      <main className={styles.mobile}>
        <div className={styles.content}>
          <Outlet />
        </div>
        {MAIN_TABS.has(pathname) ? <TabBar /> : null}
        <Toast />
      </main>
    );
  }

  return (
    <main className={styles.desktop}>
      <DesktopAside />
      <div className={styles.content}>
        <Outlet />
      </div>
    </main>
  );
}
