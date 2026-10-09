import type { ReactNode } from 'react';
import { useEffect } from 'react';
import { useLayer } from '@/store/layer';
import { errorMessage } from '@/utils/error';
import { CreateView } from './create';
import styles from './gate.module.css';
import { UnlockView } from './unlock';

type LayerProps = {
  children: ReactNode;
};

export function Layer({ children }: LayerProps) {
  const status = useLayer((state) => state.status);
  const bootError = useLayer((state) => state.bootError);
  const refresh = useLayer((state) => state.refresh);
  const failBoot = useLayer((state) => state.failBoot);

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
