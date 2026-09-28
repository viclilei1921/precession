import type { ReactNode } from 'react';
import { MemberPanel } from '@/shared/data/member/panel';
import styles from '@/shared/ui/record.module.css';

type SettingsPageProps = {
  deviceUnlock?: ReactNode;
};

export function SettingsPage({ deviceUnlock }: SettingsPageProps) {
  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>设置</h1>
        <p className={styles.hint}>数据与安全和成员档案。</p>
      </header>
      <div className={styles.card}>
        <h2 className={styles.cardTitle}>数据与安全</h2>
        {deviceUnlock}
      </div>
      <div className={styles.card}>
        <h2 className={styles.cardTitle}>成员</h2>
        <MemberPanel />
      </div>
    </section>
  );
}
