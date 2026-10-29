import { ArrowsClockwiseIcon, CopyIcon, CubeIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { arch, platform, version } from '@/bridge';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { Card } from '@/components/card';
import { appVersionsQuery } from '@/query/about/query';
import { errorMessage } from '@/utils/error';
import { CardHead } from './card-head';
import styles from './page.module.css';

type VersionsSectionProps = {
  onNotice: (message: string) => void;
};

function text(value: string | undefined, pending: boolean) {
  if (value) {
    return value;
  }
  return pending ? '正在读取…' : '未知';
}

export function VersionsSection({ onNotice }: VersionsSectionProps) {
  const versions = useQuery(appVersionsQuery);
  const rows = [
    { label: '应用', value: text(versions.data?.app, versions.isPending) },
    { label: '系统', value: `${platform()} ${version()} (${arch()})` },
    { label: 'WebView', value: text(versions.data?.webview, versions.isPending) },
    { label: 'FFmpeg', value: text(versions.data?.ffmpeg, versions.isPending) },
    { label: 'cjxl', value: text(versions.data?.cjxl, versions.isPending) },
    { label: 'avifenc', value: text(versions.data?.avifenc, versions.isPending) }
  ];

  async function copy(label: string, value: string) {
    try {
      await navigator.clipboard.writeText(`${label} ${value}`);
      onNotice(`已复制${label}`);
    } catch (err) {
      onNotice(errorMessage(err));
    }
  }

  async function copyAll() {
    const body = rows.map((row) => `${row.label} ${row.value}`).join('\n');
    try {
      await navigator.clipboard.writeText(body);
      onNotice('已复制版本信息');
    } catch (err) {
      onNotice(errorMessage(err));
    }
  }

  return (
    <Card id="versions" className={styles.section}>
      <CardHead
        tone="journal"
        icon={CubeIcon}
        title="版本"
        detail="应用与侧车工具"
        extra={versions.data?.app ? `v${versions.data.app}` : '版本'}
      />
      {versions.error ? <p className={styles.error}>{errorMessage(versions.error)}</p> : null}
      <dl className={styles.versions}>
        {rows.map((row) => (
          <div key={row.label} className={styles.version}>
            <dt className={styles.versionName}>{row.label}</dt>
            <dd className={styles.versionValue}>
              <span>{row.value}</span>
              <IconButton label={`复制${row.label}`} size="compact" onClick={() => void copy(row.label, row.value)}>
                <CopyIcon weight="regular" />
              </IconButton>
            </dd>
          </div>
        ))}
      </dl>
      <footer className={styles.foot}>
        <span className={styles.footNote}>启动时读取</span>
        <div className={styles.footActions}>
          <Button variant="text" onClick={() => void copyAll()}>
            复制信息
          </Button>
          <Button variant="outlined" disabled={versions.isFetching} onClick={() => void versions.refetch()}>
            <ArrowsClockwiseIcon weight="regular" />
            {versions.isFetching ? '正在读取…' : '重新读取'}
          </Button>
        </div>
      </footer>
    </Card>
  );
}
