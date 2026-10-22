import { useQuery } from '@tanstack/react-query';
import { arch, platform, version } from '@/bridge';
import { MemberPanel } from '@/components/member-panel';
import styles from '@/components/record.module.css';
import { Scroll } from '@/components/scroll';
import { DeviceUnlock, LockArchive } from '@/features/lock';
import { appVersionsQuery } from '@/query/about/query';
import { useMembers } from '@/query/member/use-members';
import { errorMessage } from '@/utils/error';

function versionText(value: string | undefined, pending: boolean) {
  if (value) {
    return value;
  }
  return pending ? '正在读取…' : '未知';
}

export function SettingsPage() {
  const members = useMembers();
  const versions = useQuery(appVersionsQuery);
  const system = `${platform()} ${version()} (${arch()})`;
  const rows = [
    { label: '应用', value: versionText(versions.data?.app, versions.isPending) },
    { label: '系统', value: system },
    { label: 'WebView', value: versionText(versions.data?.webview, versions.isPending) },
    { label: 'FFmpeg', value: versionText(versions.data?.ffmpeg, versions.isPending) },
    { label: 'cjxl', value: versionText(versions.data?.cjxl, versions.isPending) },
    { label: 'avifenc', value: versionText(versions.data?.avifenc, versions.isPending) }
  ];

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>设置</h1>
        <p className={styles.hint}>数据与安全、成员档案和版本。</p>
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
          <div className={styles.card}>
            <h2 className={styles.cardTitle}>版本</h2>
            {versions.error ? <p className={styles.error}>{errorMessage(versions.error)}</p> : null}
            {rows.map((row) => (
              <div key={row.label} className={styles.row}>
                <span className={styles.rowTitle}>{row.label}</span>
                <span className={styles.meta}>{row.value}</span>
              </div>
            ))}
          </div>
        </div>
      </Scroll>
    </section>
  );
}
