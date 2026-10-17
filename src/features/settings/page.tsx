import { MemberPanel } from '@/components/member-panel';
import styles from '@/components/record.module.css';
import { Scroll } from '@/components/scroll';
import { DeviceUnlock, LockArchive } from '@/features/lock';
import { useMembers } from '@/query/member/use-members';
import { errorMessage } from '@/utils/error';

export function SettingsPage() {
  const members = useMembers();

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>设置</h1>
        <p className={styles.hint}>数据与安全和成员档案。</p>
      </header>
      <Scroll className={styles.scroll}>
        <div className={styles.body}>
          <div className={styles.card}>
            <h2 className={styles.cardTitle}>数据与安全</h2>
            <LockArchive />
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
        </div>
      </Scroll>
    </section>
  );
}
