import type { ReactNode } from 'react';
import { useEffect, useState } from 'react';
import { dbStatus } from '@/bridge/db';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import { CreateView } from './create';
import styles from './gate.module.css';
import { UnlockView } from './unlock';

type LayerProps = {
  children: ReactNode;
};

export function Layer({ children }: LayerProps) {
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

  if (!status.exists) {
    return <CreateView onDone={refresh} />;
  }

  if (!status.unlocked) {
    return <UnlockView deviceUnlock={status.deviceUnlock} userLocked={status.userLocked} onDone={refresh} />;
  }

  return children;
}
