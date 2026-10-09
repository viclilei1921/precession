import { QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from '@tanstack/react-router';
import { Layer } from '@/layout/layer';
import { TitleBar } from '@/layout/titlebar';
import { queryClient } from '@/query/client';
import { router } from '@/router';
import styles from './app.module.css';

export default function App() {
  return (
    <div className={styles.app}>
      <TitleBar />
      <main className={styles.main}>
        <Layer>
          <QueryClientProvider client={queryClient}>
            <RouterProvider router={router} />
          </QueryClientProvider>
        </Layer>
      </main>
    </div>
  );
}
