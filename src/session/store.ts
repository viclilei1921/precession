import { create } from 'zustand';
import type { DbStatus } from '@/bridge/db';
import { dbLock, dbStatus } from '@/bridge/db';
import { reset as resetGrowth } from '@/features/growth/reset';
import { reset as resetJournal } from '@/features/journal/reset';
import { reset as resetLibrary } from '@/features/library/reset';
import { reset as resetPlan } from '@/features/plan/reset';
import { reset as resetReview } from '@/features/review/reset';
import { reset as resetSettings } from '@/features/settings/reset';
import { reset as resetToday } from '@/features/today/reset';
import { reset as resetToolbox } from '@/features/toolbox/reset';
import { queryClient } from './query';

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
    resetToday();
    resetPlan();
    resetJournal();
    resetGrowth();
    resetLibrary();
    resetToolbox();
    resetReview();
    resetSettings();
    const status = await dbStatus();
    set({ status });
  }
}));
