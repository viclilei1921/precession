import { create } from 'zustand';

type TitleBarActions = {
  active: boolean;
  captureOpen: boolean;
  commandOpen: boolean;
  setActive: (active: boolean) => void;
  openCapture: () => void;
  closeCapture: () => void;
  openCommand: () => void;
  closeCommand: () => void;
};

export const useTitleBarActions = create<TitleBarActions>((set) => ({
  active: false,
  captureOpen: false,
  commandOpen: false,
  setActive: (active) => set(active ? { active } : { active: false, captureOpen: false, commandOpen: false }),
  openCapture: () => set({ captureOpen: true }),
  closeCapture: () => set({ captureOpen: false }),
  openCommand: () => set({ commandOpen: true }),
  closeCommand: () => set({ commandOpen: false })
}));
