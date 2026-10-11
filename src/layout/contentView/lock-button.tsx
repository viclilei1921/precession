import { LockIcon } from '@phosphor-icons/react';
import { useState } from 'react';
import { dbLock, dbStatus } from '@/bridge/db';
import { queryClient } from '@/query/client';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import styles from './lock-button.module.css';

export function LockButton() {
  const setDbStatus = useAppStore((state) => state.setDbStatus);
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
    <div className={styles.lock}>
      {error ? <p className={styles.error}>{error}</p> : null}
      <button type="button" className={styles.button} disabled={locking} onClick={() => void lockArchive()}>
        <LockIcon className={styles.icon} weight="regular" />
        {locking ? '正在锁定…' : '锁定'}
      </button>
    </div>
  );
}
