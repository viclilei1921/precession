import { useEffect, useState } from 'react';
import { dbStatus } from '@/bridge/db';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import { CreateView } from './create';
import styles from './index.module.css';
import { UnlockView } from './unlock';

/** 解锁前的页面：加载状态、创建档案和解锁。 */
export function LockView() {
  const status = useAppStore((state) => state.dbStatus);
  const setDbStatus = useAppStore((state) => state.setDbStatus);
  const [bootError, setBootError] = useState('');

  async function refresh() {
    const next = await dbStatus();
    setDbStatus(next);
    setBootError('');
    return next;
  }

  useEffect(() => {
    let cancelled = false;
    dbStatus()
      .then((next) => {
        if (!cancelled) {
          setDbStatus(next);
          setBootError('');
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setBootError(errorMessage(error));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [setDbStatus]);

  if (bootError) {
    return (
      <div className={styles.gate}>
        <h1>人生档案</h1>
        <p className={styles.error}>{bootError}</p>
      </div>
    );
  }

  if (!status) {
    return (
      <div className={styles.gate}>
        <h1>人生档案</h1>
        <p className={styles.muted}>加载中…</p>
      </div>
    );
  }

  if (status.unlocked) {
    return null;
  }

  if (!status.exists) {
    return <CreateView onDone={refresh} />;
  }

  return <UnlockView deviceUnlock={status.deviceUnlock} userLocked={status.userLocked} onDone={refresh} />;
}
