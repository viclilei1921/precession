import { ArrowRightIcon, CopyIcon, DownloadSimpleIcon, ShieldCheckIcon, UploadSimpleIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
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
  const setDbStatus = useAppStore((state) => state.setDbStatus);
  const info = useQuery(archiveInfoQuery);
  const [busy, setBusy] = useState<'export' | 'import' | ''>('');
  const [pending, setPending] = useState('');
  const [error, setError] = useState('');

  const fileHint = info.data
    ? `${info.data.fileCount} 个 · ${formatBytes(info.data.fileBytes)} · 记录上的附件都存在本机。`
    : info.isPending
      ? '正在读取…'
      : '记录上的附件都存在本机。';

  async function copyPath() {
    if (!info.data?.path) {
      return;
    }
    try {
      await navigator.clipboard.writeText(info.data.path);
      onNotice('已复制档案路径');
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
      const name = packed.path.split(/[/\\]/).pop() || '档案';
      onNotice(`已导出 ${name} · ${formatBytes(packed.bytes)}`);
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
      <CardHead tone="plan" icon={ShieldCheckIcon} title="数据与安全" detail="档案锁定与设备解锁" extra="仅本机" />
      <div className={styles.rows}>
        <LockArchive />
        <DeviceUnlock />
        <SettingRow title="文件" hint={fileHint}>
          <LinkButton to={PATH.files} variant="text">
            打开文件
            <ArrowRightIcon weight="regular" />
          </LinkButton>
        </SettingRow>
        <SettingRow title="导入导出" hint="把数据库和全部文件打成一个压缩包，或从压缩包恢复。">
          <Button variant="outlined" disabled={busy !== ''} onClick={() => void onExport()}>
            <DownloadSimpleIcon weight="regular" />
            {busy === 'export' ? '正在导出…' : '导出'}
          </Button>
          <Button variant="outlined" disabled={busy !== ''} onClick={() => void chooseImport()}>
            <UploadSimpleIcon weight="regular" />
            {busy === 'import' ? '正在导入…' : '导入'}
          </Button>
        </SettingRow>
        <SettingRow
          title="档案位置"
          hint={<code className={styles.path}>{info.data?.path ?? (info.isPending ? '正在读取…' : '未知')}</code>}
          hintTitle={info.data?.path}
        >
          <IconButton label="复制档案路径" size="compact" disabled={!info.data} onClick={() => void copyPath()}>
            <CopyIcon weight="regular" />
          </IconButton>
        </SettingRow>
      </div>
      {info.error ? <p className={styles.error}>{errorMessage(info.error)}</p> : null}
      {!pending && error ? <p className={styles.error}>{error}</p> : null}
      {pending ? (
        <Modal
          title="导入档案？"
          onClose={() => {
            if (busy !== 'import') {
              setPending('');
            }
          }}
        >
          <p className={styles.dialogBody}>
            这会用所选压缩包里的数据库和文件替换这台设备上的内容。设备解锁会关闭，之后要用这份档案的密码重新打开。
          </p>
          {error ? <p className={styles.error}>{error}</p> : null}
          <div className={styles.dialogActions}>
            <Button variant="text" disabled={busy === 'import'} onClick={() => setPending('')}>
              取消
            </Button>
            <Button disabled={busy === 'import'} onClick={() => void confirmImport()}>
              {busy === 'import' ? '正在导入…' : '导入'}
            </Button>
          </div>
        </Modal>
      ) : null}
    </Card>
  );
}
