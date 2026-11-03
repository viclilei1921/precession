import { SidebarSimpleIcon } from '@phosphor-icons/react';
import { useTranslation } from 'react-i18next';
import { IconButton } from '@/components/button/icon';
import { useAppStore } from '@/store/app';

/** 标题栏上的侧栏展开、收起。 */
export function TitleBarAsideToggle() {
  const { t } = useTranslation('layout');
  const collapsed = useAppStore((state) => state.sidebarCollapsed);
  const toggleSidebar = useAppStore((state) => state.toggleSidebar);
  const label = collapsed ? t('titleBar.expandSidebar') : t('titleBar.collapseSidebar');

  return (
    <IconButton label={label} size="compact" selected={collapsed} expanded={!collapsed} onClick={toggleSidebar}>
      <SidebarSimpleIcon weight="regular" />
    </IconButton>
  );
}
