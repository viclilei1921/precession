import styles from './page.module.css';

export function SettingsPage() {
  return (
    <section className={styles.page}>
      <h1 className={styles.title}>设置</h1>
      <p className={styles.hint}>数据与安全、成员、外观和快捷键会放在这里。</p>
    </section>
  );
}
