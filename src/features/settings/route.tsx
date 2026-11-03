import { useAppStore } from '@/store/app';
import { MobileSettingsPage } from './mobile/page';
import { SettingsPage } from './page';

/** 设置页分流。桌面长页留在 page.tsx。 */
export function SettingsRoute() {
  const mobile = useAppStore((state) => state.isMobile());
  return mobile ? <MobileSettingsPage /> : <SettingsPage />;
}
