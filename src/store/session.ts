import { create } from 'zustand';
import type { DbStatus } from '@/bridge/db';
import { dbLock, dbStatus } from '@/bridge/db';
import { queryClient } from '@/session/query-client';

type SessionStore = {
  status: DbStatus | null;
  bootError: string;
  refresh: () => Promise<DbStatus>;
  lock: () => Promise<void>;
  failBoot: (message: string) => void;
};

export const useSession = create<SessionStore>((set) => ({
  status: null,
  bootError: '',
  failBoot: (bootError) => set({ bootError }),
  refresh: async () => {
    const status = await dbStatus();
    set({ status });
    return status;
  },
  lock: async () => {
    await dbLock();
    queryClient.clear();
    const status = await dbStatus();
    set({ status });
  }
}));
