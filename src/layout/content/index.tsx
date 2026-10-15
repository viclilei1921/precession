import { QueryClientProvider } from '@tanstack/react-query';
import { Outlet, RouterProvider } from '@tanstack/react-router';
import { queryClient } from '@/query/client';
import { router } from '@/router';
import { useAppStore } from '@/store/app';
import { DesktopAside } from './aside/desktop';
import { MobileAside } from './aside/mobile';
import styles from './index.module.css';

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

  if (mobile) {
    return (
      <div className={styles.mobile}>
        <div className={styles.content}>
          <Outlet />
        </div>
        <MobileAside />
      </div>
    );
  }

  return (
    <div className={styles.desktop}>
      <DesktopAside />
      <div className={styles.main}>
        <div className={styles.content}>
          <Outlet />
        </div>
      </div>
    </div>
  );
}
