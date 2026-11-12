import { QueryClientProvider } from '@tanstack/react-query';
import { Outlet, RouterProvider, useNavigate, useRouterState } from '@tanstack/react-router';
import { useEffect } from 'react';
import { peekViewerPath, subscribeViewerPath } from '@/features/viewer/open';
import { MobileBackBridge } from '@/layout/mobile/back';
import { TabBar } from '@/layout/mobile/tab-bar';
import { Toast } from '@/layout/mobile/toast';
import { queryClient } from '@/query/client';
import { router } from '@/router';
import { PATH } from '@/router/path';
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

/** 系统打开图片时，解锁后进入看图。 */
function ViewerOpenBridge() {
  const navigate = useNavigate();

  useEffect(() => {
    const open = () => {
      void navigate({ to: PATH.viewer });
    };
    if (peekViewerPath()) {
      open();
    }
    return subscribeViewerPath(open);
  }, [navigate]);

  return null;
}

/** 路由根布局：侧栏和功能区出口。 */
export function ContentLayout() {
  const mobile = useAppStore((state) => state.isMobile());
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const bleed = pathname === PATH.viewer ? 'true' : undefined;

  if (mobile) {
    return (
      <main className={styles.mobile}>
        <ViewerOpenBridge />
        <MobileBackBridge />
        <div className={styles.content} data-bleed={bleed}>
          <Outlet />
        </div>
        {MAIN_TABS.has(pathname) ? <TabBar /> : null}
        <Toast />
      </main>
    );
  }

  return (
    <main className={styles.desktop}>
      <ViewerOpenBridge />
      <DesktopAside />
      <div className={styles.content} data-bleed={bleed}>
        <Outlet />
      </div>
    </main>
  );
}
