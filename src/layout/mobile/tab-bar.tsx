import { Link } from '@tanstack/react-router';
import { mobileNavItems } from '@/router/items';
import styles from './tab-bar.module.css';

/** 移动端底栏：计划、手记、成长、书库、设置。 */
export function TabBar() {
  return (
    <nav className={styles.tabbar} aria-label="主导航">
      {mobileNavItems().map((item) => (
        <Link
          key={item.id}
          to={item.to}
          className={styles.tab}
          data-tone={item.tone}
          activeOptions={{ exact: item.to === '/library' }}
        >
          <item.icon className={styles.icon} weight="regular" />
          {item.label}
        </Link>
      ))}
    </nav>
  );
}
