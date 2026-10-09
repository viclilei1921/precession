import { create } from 'zustand';
import type { DbStatus } from '@/bridge/db';
import { dbLock, dbStatus } from '@/bridge/db';
import { queryClient } from '@/query/client';

type LayerState = {
  status: DbStatus | null;
  bootError: string;
};

type LayerActions = {
  refresh: () => Promise<DbStatus>;
  lock: () => Promise<void>;
  failBoot: (message: string) => void;
};

export const useLayerStore = create<LayerState & LayerActions>()((set, get) => {
  let epoch = 0;

  return {
    status: null,
    bootError: '',
    failBoot: (bootError) => set({ bootError }),
    refresh: async () => {
      const ticket = ++epoch;
      try {
        const status = await dbStatus();
        if (ticket === epoch) {
          set({ status, bootError: '' });
        }
        return status;
      } catch (error) {
        const current = get().status;
        if (ticket !== epoch && current) {
          return current;
        }
        throw error;
      }
    },
    lock: async () => {
      await dbLock();
      queryClient.clear();
      await get().refresh();
    }
  };
});
