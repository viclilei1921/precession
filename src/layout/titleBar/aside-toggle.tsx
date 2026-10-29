import { SidebarSimpleIcon } from '@phosphor-icons/react';
import { IconButton } from '@/components/button/icon';
import { useAppStore } from '@/store/app';

/** 标题栏上的侧栏展开、收起。 */
export function TitleBarAsideToggle() {
  const collapsed = useAppStore((state) => state.sidebarCollapsed);
  const toggleSidebar = useAppStore((state) => state.toggleSidebar);
  const label = collapsed ? '展开侧栏' : '收起侧栏';

  return (
    <IconButton label={label} size="compact" selected={collapsed} expanded={!collapsed} onClick={toggleSidebar}>
      <SidebarSimpleIcon weight="regular" />
    </IconButton>
  );
}
