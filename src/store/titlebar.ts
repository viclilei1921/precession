import { create } from 'zustand';

type TitleBarActions = {
  active: boolean;
  captureOpen: boolean;
  query: string;
  placeholder: string;
  focusTick: number;
  setActive: (active: boolean) => void;
  openCapture: () => void;
  closeCapture: () => void;
  setQuery: (query: string) => void;
  registerSearch: (placeholder: string) => () => void;
  requestFocus: () => void;
};

export const useTitleBarActions = create<TitleBarActions>((set) => ({
  active: false,
  captureOpen: false,
  query: '',
  placeholder: '',
  focusTick: 0,
  setActive: (active) => set(active ? { active } : { active: false, captureOpen: false, query: '', placeholder: '' }),
  openCapture: () => set({ captureOpen: true }),
  closeCapture: () => set({ captureOpen: false }),
  setQuery: (query) => set({ query }),
  registerSearch: (placeholder) => {
    set({ placeholder, query: '' });
    return () => set({ placeholder: '', query: '' });
  },
  requestFocus: () => set((state) => ({ focusTick: state.focusTick + 1 }))
}));
