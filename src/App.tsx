import { QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from '@tanstack/react-router';
import { router } from '@/router';
import { SessionGate } from '@/session/gate';
import { queryClient } from '@/session/query-client';

export default function App() {
  return (
    <SessionGate>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </SessionGate>
  );
}
