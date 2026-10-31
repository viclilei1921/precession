import { useAppStore } from '@/store/app';
import { MobilePlanPage } from './mobile/page';
import { PlanPage } from './page';

/** 计划页分流。桌面实现留在 page.tsx。 */
export function PlanRoute() {
  const mobile = useAppStore((state) => state.isMobile());
  return mobile ? <MobilePlanPage /> : <PlanPage />;
}
