import type { ReactNode } from 'react';
import { useEffect } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { useLayerStore } from '@/store/layer';
import { errorMessage } from '@/utils/error';
import { CreateView } from './create';
import styles from './gate.module.css';
import { UnlockView } from './unlock';

type LayerProps = {
  children: ReactNode;
};

export function Layer({ children }: LayerProps) {
  const { status, bootError, refresh, failBoot } = useLayerStore(
    useShallow((state) => ({
      status: state.status,
      bootError: state.bootError,
      refresh: state.refresh,
      failBoot: state.failBoot
    }))
  );

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
