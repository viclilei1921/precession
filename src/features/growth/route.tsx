import { useAppStore } from '@/store/app';
import { MobileGrowthPage } from './mobile/page';
import { GrowthPage } from './page';

/** 成长页分流。桌面实现留在 page.tsx。 */
export function GrowthRoute() {
  const mobile = useAppStore((state) => state.isMobile());
  return mobile ? <MobileGrowthPage /> : <GrowthPage />;
}
