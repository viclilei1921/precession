import { QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from '@tanstack/react-router';
import { useEffect } from 'react';
import { errorMessage } from './error';
import { CreateView } from './gate/CreateView';
import styles from './gate/gate.module.css';
import { UnlockView } from './gate/UnlockView';
import { queryClient } from './query';
import { router } from './router';
import { useSession } from './session';

export default function App() {
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
    return <UnlockView deviceUnlock={status.deviceUnlock} held={status.userLocked} onDone={refresh} />;
  }

  return (
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}
