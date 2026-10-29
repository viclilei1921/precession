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

  /** 启动错误 */
  const [bootError, setBootError] = useState('');

  /** 刷新数据库状态 */
  async function refresh() {
    const next = await dbStatus();
    setDbStatus(next);
    setBootError('');
    return next;
  }

  useEffect(() => {
    let cancelled = false;
    const { setDbStatus } = useAppStore.getState();

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
  }, []);

  if (bootError) {
    return (
      <main className={styles.gate}>
        <h1>人生档案</h1>
        <p className={styles.error}>{bootError}</p>
      </main>
    );
  }

  if (!status) {
    return (
      <main className={styles.gate}>
        <h1>人生档案</h1>
        <p className={styles.muted}>加载中…</p>
      </main>
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
