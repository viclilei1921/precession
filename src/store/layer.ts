import { create } from 'zustand';
import type { DbStatus } from '@/bridge/db';
import { dbLock, dbStatus } from '@/bridge/db';
import { queryClient } from '@/query/client';

type LayerStore = {
  status: DbStatus | null;
  bootError: string;
  refresh: () => Promise<DbStatus>;
  lock: () => Promise<void>;
  failBoot: (message: string) => void;
};

export const useLayer = create<LayerStore>((set) => ({
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
