import { Link } from '@tanstack/react-router';
import { useTranslation } from 'react-i18next';
import { mobileNavItems } from '@/router/items';
import styles from './tab-bar.module.css';

/** 移动端底栏：计划、手记、成长、书库、设置。 */
export function TabBar() {
  const { t } = useTranslation('nav');

  return (
    <nav className={styles.tabbar} aria-label={t('group.record')}>
      {mobileNavItems().map((item) => (
        <Link
          key={item.id}
          to={item.to}
          replace
          className={styles.tab}
          data-tone={item.tone}
          activeOptions={{ exact: item.to === '/library' }}
        >
          <item.icon className={styles.icon} weight="regular" />
          {t(item.id)}
        </Link>
      ))}
    </nav>
  );
}
