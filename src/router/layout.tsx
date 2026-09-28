import { Link, Outlet } from '@tanstack/react-router';
import { SessionControls } from '@/session/controls';
import styles from './layout.module.css';
import { PATH } from './path';

const recordLinks = [
  [PATH.today, '今天'],
  [PATH.plan, '计划'],
  [PATH.journal, '手记'],
  [PATH.growth, '成长'],
  [PATH.library, '书库']
] as const;

const organizeLinks = [
  [PATH.toolbox, '工具箱'],
  [PATH.review, '回顾']
] as const;

export function Layout() {
  return (
    <div className={styles.layout}>
      <aside className={styles.nav}>
        <p className={styles.brand}>Precession</p>
        <nav className={styles.group} aria-label="记录">
          <div className={styles.label}>记录</div>
          {recordLinks.map(([to, label]) => (
            <Link key={to} to={to} className={styles.link}>
              {label}
            </Link>
          ))}
        </nav>
        <nav className={styles.group} aria-label="整理">
          <div className={styles.label}>整理</div>
          {organizeLinks.map(([to, label]) => (
            <Link key={to} to={to} className={styles.link}>
              {label}
            </Link>
          ))}
        </nav>
        <nav className={styles.group} aria-label="系统">
          <div className={styles.label}>系统</div>
          <Link to={PATH.settings} className={styles.link}>
            设置
          </Link>
        </nav>
        <SessionControls />
      </aside>
      <div className={styles.main}>
        <header className={styles.topbar}>
          <input className={styles.search} readOnly placeholder="搜索全部记录" aria-label="搜索全部记录" />
          <button type="button" className={styles.capture}>
            记一笔
          </button>
        </header>
        <div className={styles.content}>
          <Outlet />
        </div>
      </div>
    </div>
  );
}
