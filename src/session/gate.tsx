import type { ReactNode } from 'react';
import { useEffect } from 'react';
import { errorMessage } from '@/shared/lib/error';
import { CreateView } from './create';
import styles from './gate.module.css';
import { useSession } from './store';
import { UnlockView } from './unlock';

type SessionGateProps = {
  children: ReactNode;
};

export function SessionGate({ children }: SessionGateProps) {
  const status = useSession((state) => state.status);
  const bootError = useSession((state) => state.bootError);
  const refresh = useSession((state) => state.refresh);
  const failBoot = useSession((state) => state.failBoot);

  useEffect(() => {
    let cancelled = false;
    refresh().catch((error: unknown) => {
      if (!cancelled) {
        failBoot(errorMessage(error));
      }
    });
    return () => {
      cancelled = true;
    };
  }, [refresh, failBoot]);

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
