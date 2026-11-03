import { LockIcon } from '@phosphor-icons/react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { dbLock, dbStatus } from '@/bridge/db';
import { Button } from '@/components/button';
import { Modal } from '@/components/modal';
import { SettingRow } from '@/features/settings/row';
import { queryClient } from '@/query/client';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import styles from './lock-archive.module.css';

/** 确认后锁上档案并回到锁屏。 */
export function LockArchive() {
  const { t } = useTranslation('lock');
  const setDbStatus = useAppStore((state) => state.setDbStatus);
  const [open, setOpen] = useState(false);
  const [locking, setLocking] = useState(false);
  const [error, setError] = useState('');

  async function lockArchive() {
    setLocking(true);
    setError('');
    try {
      await dbLock();
      queryClient.clear();
      setDbStatus(await dbStatus());
    } catch (err) {
      setError(errorMessage(err));
      setLocking(false);
    }
  }

  return (
    <>
      <SettingRow title={t('archive.rowTitle')} hint={t('archive.rowHint')}>
        <Button variant="outlined" disabled={locking} onClick={() => setOpen(true)}>
          <LockIcon weight="regular" />
          {t('archive.action')}
        </Button>
      </SettingRow>
      {open ? (
        <Modal title={t('archive.modalTitle')} onClose={() => setOpen(false)}>
          <p className={styles.body}>{t('archive.modalBody')}</p>
          {error ? <p className={styles.error}>{error}</p> : null}
          <div className={styles.actions}>
            <Button variant="text" disabled={locking} onClick={() => setOpen(false)}>
              {t('archive.cancel')}
            </Button>
            <Button disabled={locking} onClick={() => void lockArchive()}>
              {locking ? t('archive.locking') : t('archive.action')}
            </Button>
          </div>
        </Modal>
      ) : null}
    </>
  );
}
