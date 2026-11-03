import { ArrowRightIcon, CopyIcon, DownloadSimpleIcon, ShieldCheckIcon, UploadSimpleIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { archiveExport, archiveImport, pickArchiveOpen, pickArchiveSave } from '@/bridge/archive';
import { dbStatus } from '@/bridge/db';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { LinkButton } from '@/components/button/link';
import { Card } from '@/components/card';
import { Modal } from '@/components/modal';
import { DeviceUnlock, LockArchive } from '@/features/lock';
import { archiveInfoQuery } from '@/query/archive/query';
import { queryClient } from '@/query/client';
import { PATH } from '@/router/path';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import { formatBytes } from './bytes';
import { CardHead } from './card-head';
import styles from './page.module.css';
import { SettingRow } from './row';

type SecuritySectionProps = {
  onNotice: (message: string) => void;
};

export function SecuritySection({ onNotice }: SecuritySectionProps) {
  const { t } = useTranslation('settings');
  const setDbStatus = useAppStore((state) => state.setDbStatus);
  const info = useQuery(archiveInfoQuery);
  const [busy, setBusy] = useState<'export' | 'import' | ''>('');
  const [pending, setPending] = useState('');
  const [error, setError] = useState('');

  const fileHint = info.data
    ? t('security.fileRow.hintFormat', { count: info.data.fileCount, size: formatBytes(info.data.fileBytes) })
    : info.isPending
      ? t('security.fileRow.hintLoading')
      : t('security.fileRow.hintDefault');

  async function copyPath() {
    if (!info.data?.path) {
      return;
    }
    try {
      await navigator.clipboard.writeText(info.data.path);
      onNotice(t('security.noticeCopied'));
    } catch (err) {
      setError(errorMessage(err));
    }
  }

  async function onExport() {
    setError('');
    const path = await pickArchiveSave();
    if (!path) {
      return;
    }
    setBusy('export');
    try {
      const packed = await archiveExport(path);
      const name = packed.path.split(/[/\\]/).pop() || t('security.defaultArchiveName');
      onNotice(t('security.noticeExported', { name, size: formatBytes(packed.bytes) }));
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy('');
    }
  }

  async function chooseImport() {
    setError('');
    const path = await pickArchiveOpen();
    if (path) {
      setPending(path);
    }
  }

  async function confirmImport() {
    if (!pending) {
      return;
    }
    setBusy('import');
    setError('');
    try {
      await archiveImport(pending);
      queryClient.clear();
      setDbStatus(await dbStatus());
    } catch (err) {
      setError(errorMessage(err));
      setBusy('');
    }
  }

  return (
    <Card id="security" className={styles.section}>
      <CardHead
        tone="plan"
        icon={ShieldCheckIcon}
        title={t('security.title')}
        detail={t('security.detail')}
        extra={t('security.extra')}
      />
      <div className={styles.rows}>
        <LockArchive />
        <DeviceUnlock />
        <SettingRow title={t('security.fileRow.title')} hint={fileHint}>
          <LinkButton to={PATH.files} variant="text">
            {t('security.fileRow.open')}
            <ArrowRightIcon weight="regular" />
          </LinkButton>
        </SettingRow>
        <SettingRow title={t('security.ioRow.title')} hint={t('security.ioRow.hint')}>
          <Button variant="outlined" disabled={busy !== ''} onClick={() => void onExport()}>
            <DownloadSimpleIcon weight="regular" />
            {busy === 'export' ? t('security.ioRow.exporting') : t('security.ioRow.export')}
          </Button>
          <Button variant="outlined" disabled={busy !== ''} onClick={() => void chooseImport()}>
            <UploadSimpleIcon weight="regular" />
            {busy === 'import' ? t('security.ioRow.importing') : t('security.ioRow.import')}
          </Button>
        </SettingRow>
        <SettingRow
          title={t('security.pathRow.title')}
          hint={
            <code className={styles.path}>
              {info.data?.path ?? (info.isPending ? t('common.loading') : t('security.pathRow.unknown'))}
            </code>
          }
          hintTitle={info.data?.path}
        >
          <IconButton
            label={t('security.pathRow.copy')}
            size="compact"
            disabled={!info.data}
            onClick={() => void copyPath()}
          >
            <CopyIcon weight="regular" />
          </IconButton>
        </SettingRow>
      </div>
      {info.error ? <p className={styles.error}>{errorMessage(info.error)}</p> : null}
      {!pending && error ? <p className={styles.error}>{error}</p> : null}
      {pending ? (
        <Modal
          title={t('security.importModal.title')}
          onClose={() => {
            if (busy !== 'import') {
              setPending('');
            }
          }}
        >
          <p className={styles.dialogBody}>{t('security.importModal.body')}</p>
          {error ? <p className={styles.error}>{error}</p> : null}
          <div className={styles.dialogActions}>
            <Button variant="text" disabled={busy === 'import'} onClick={() => setPending('')}>
              {t('security.importModal.cancel')}
            </Button>
            <Button disabled={busy === 'import'} onClick={() => void confirmImport()}>
              {busy === 'import' ? t('security.importModal.importing') : t('security.importModal.confirm')}
            </Button>
          </div>
        </Modal>
      ) : null}
    </Card>
  );
}
