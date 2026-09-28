import type { ReactNode } from 'react';
import styles from '@/shared/ui/page.module.css';

type SettingsPageProps = {
  deviceUnlock?: ReactNode;
};

export function SettingsPage({ deviceUnlock }: SettingsPageProps) {
  return (
    <section className={styles.page}>
      <h1 className={styles.title}>设置</h1>
      <p className={styles.hint}>数据与安全、成员、外观和快捷键会放在这里。</p>
      {deviceUnlock}
    </section>
  );
}
