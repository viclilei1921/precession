import { QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from '@tanstack/react-router';
import { TitleBar } from '@/layout/titlebar';
import { TitleBarActionsProvider } from '@/layout/titlebar-actions';
import { router } from '@/router';
import { SessionGate } from '@/session/gate';
import { queryClient } from '@/session/query-client';
import styles from './app.module.css';

export default function App() {
  return (
    <TitleBarActionsProvider>
      <div className={styles.app}>
        <TitleBar />
        <main className={styles.main}>
          <SessionGate>
            <QueryClientProvider client={queryClient}>
              <RouterProvider router={router} />
            </QueryClientProvider>
          </SessionGate>
        </main>
      </div>
    </TitleBarActionsProvider>
  );
}
