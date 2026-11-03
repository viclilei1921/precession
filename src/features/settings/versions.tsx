import { ArrowsClockwiseIcon, CopyIcon, CubeIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
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

export function VersionsSection({ onNotice }: VersionsSectionProps) {
  const { t } = useTranslation('settings');
  const versions = useQuery(appVersionsQuery);

  function text(value: string | undefined, pending: boolean) {
    if (value) {
      return value;
    }
    return pending ? t('versions.loading') : t('versions.unknown');
  }

  const rows = [
    { key: 'app', label: t('versions.app'), value: text(versions.data?.app, versions.isPending) },
    { key: 'system', label: t('versions.system'), value: `${platform()} ${version()} (${arch()})` },
    { key: 'webview', label: t('versions.webview'), value: text(versions.data?.webview, versions.isPending) },
    { key: 'ffmpeg', label: t('versions.ffmpeg'), value: text(versions.data?.ffmpeg, versions.isPending) },
    { key: 'cjxl', label: t('versions.cjxl'), value: text(versions.data?.cjxl, versions.isPending) },
    { key: 'avifenc', label: t('versions.avifenc'), value: text(versions.data?.avifenc, versions.isPending) }
  ];

  async function copy(label: string, value: string) {
    try {
      await navigator.clipboard.writeText(`${label} ${value}`);
      onNotice(t('versions.noticeCopied', { label }));
    } catch (err) {
      onNotice(errorMessage(err));
    }
  }

  async function copyAll() {
    const body = rows.map((row) => `${row.label} ${row.value}`).join('\n');
    try {
      await navigator.clipboard.writeText(body);
      onNotice(t('versions.noticeCopiedAll'));
    } catch (err) {
      onNotice(errorMessage(err));
    }
  }

  return (
    <Card id="versions" className={styles.section}>
      <CardHead
        tone="journal"
        icon={CubeIcon}
        title={t('versions.title')}
        detail={t('versions.detail')}
        extra={versions.data?.app ? `v${versions.data.app}` : t('versions.extraDefault')}
      />
      {versions.error ? <p className={styles.error}>{errorMessage(versions.error)}</p> : null}
      <dl className={styles.versions}>
        {rows.map((row) => (
          <div key={row.key} className={styles.version}>
            <dt className={styles.versionName}>{row.label}</dt>
            <dd className={styles.versionValue}>
              <span>{row.value}</span>
              <IconButton
                label={t('versions.copyRow', { label: row.label })}
                size="compact"
                onClick={() => void copy(row.label, row.value)}
              >
                <CopyIcon weight="regular" />
              </IconButton>
            </dd>
          </div>
        ))}
      </dl>
      <footer className={styles.foot}>
        <span className={styles.footNote}>{t('versions.footNote')}</span>
        <div className={styles.footActions}>
          <Button variant="text" onClick={() => void copyAll()}>
            {t('versions.copyAll')}
          </Button>
          <Button variant="outlined" disabled={versions.isFetching} onClick={() => void versions.refetch()}>
            <ArrowsClockwiseIcon weight="regular" />
            {versions.isFetching ? t('versions.loading') : t('versions.refetch')}
          </Button>
        </div>
      </footer>
    </Card>
  );
}
