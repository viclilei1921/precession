import { MemberPanel } from '@/components/member-panel';
import styles from '@/components/record.module.css';
import { useMembers } from '@/query/member/use-members';
import { errorMessage } from '@/utils/error';
import { DeviceUnlock } from './device-unlock';

export function SettingsPage() {
  const members = useMembers();

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>设置</h1>
        <p className={styles.hint}>数据与安全和成员档案。</p>
      </header>
      <div className={styles.card}>
        <h2 className={styles.cardTitle}>数据与安全</h2>
        <DeviceUnlock />
      </div>
      <div className={styles.card}>
        <h2 className={styles.cardTitle}>成员</h2>
        <MemberPanel
          members={members.members}
          listError={members.error ? errorMessage(members.error) : ''}
          isPending={members.isPending}
          onSave={members.save}
          onDelete={members.remove}
        />
      </div>
    </section>
  );
}
