import { Link } from '@tanstack/react-router';
import { mobileNavItems } from '@/router/items';
import styles from './index.module.css';

/** 移动端底栏：计划、手记、成长、书库、设置。 */
export function MobileAside() {
  return (
    <nav className={styles.tabbar} aria-label="主导航">
      {mobileNavItems().map((item) => (
        <Link key={item.id} to={item.to} className={styles.tab} activeOptions={{ exact: false }}>
          <item.icon className={styles.icon} weight="regular" />
          {item.label}
        </Link>
      ))}
    </nav>
  );
}
