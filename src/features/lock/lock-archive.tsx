import { LockIcon } from '@phosphor-icons/react';
import { useState } from 'react';
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
      <SettingRow title="档案" hint="已解锁 · 锁定后回到锁屏。">
        <Button variant="outlined" disabled={locking} onClick={() => setOpen(true)}>
          <LockIcon weight="regular" />
          锁定档案
        </Button>
      </SettingRow>
      {open ? (
        <Modal title="锁定档案？" onClose={() => setOpen(false)}>
          <p className={styles.body}>
            锁定后会回到锁屏。已启用设备解锁时，再打开需要系统验证；否则要重新输入档案密码。
          </p>
          {error ? <p className={styles.error}>{error}</p> : null}
          <div className={styles.actions}>
            <Button variant="text" disabled={locking} onClick={() => setOpen(false)}>
              取消
            </Button>
            <Button disabled={locking} onClick={() => void lockArchive()}>
              {locking ? '正在锁定…' : '锁定档案'}
            </Button>
          </div>
        </Modal>
      ) : null}
    </>
  );
}
