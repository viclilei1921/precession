import { create } from 'zustand';

type TitleBarState = {
  active: boolean;
  captureOpen: boolean;
  query: string;
  placeholder: string;
  focusTick: number;
};

type TitleBarActions = {
  setActive: (active: boolean) => void;
  openCapture: () => void;
  closeCapture: () => void;
  setQuery: (query: string) => void;
  openSearch: (placeholder: string) => void;
  closeSearch: () => void;
  requestFocus: () => void;
};

const idleSearch = {
  query: '',
  placeholder: ''
};

export const useTitleBarStore = create<TitleBarState & TitleBarActions>()((set) => ({
  active: false,
  captureOpen: false,
  ...idleSearch,
  focusTick: 0,
  setActive: (active) => set(active ? { active: true } : { active: false, captureOpen: false, ...idleSearch }),
  openCapture: () => set({ captureOpen: true }),
  closeCapture: () => set({ captureOpen: false }),
  setQuery: (query) => set({ query }),
  openSearch: (placeholder) => set({ placeholder, query: '' }),
  closeSearch: () => set(idleSearch),
  requestFocus: () => set((state) => ({ focusTick: state.focusTick + 1 }))
}));
