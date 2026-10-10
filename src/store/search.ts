import { create } from 'zustand';

type SearchState = {
  /** 是否显示搜索框 */
  show: boolean;
  /** 搜索框占位文案 */
  placeholder: string;
  /** 搜索框的值 */
  value: string;
  /** 递增后让搜索框获得焦点 */
  focusTick: number;
};

type SearchActions = {
  /** 显示搜索框 */
  setShow: (show: boolean) => void;
  /** 设置搜索框占位文案 */
  setPlaceholder: (placeholder: string) => void;
  /** 设置搜索框的值 */
  setValue: (value: string) => void;
  /** 请求搜索框获得焦点 */
  requestFocus: () => void;
};

/** 搜索状态 */
export const useSearchStore = create<SearchState & SearchActions>((set) => ({
  show: false,
  placeholder: '',
  value: '',
  focusTick: 0,
  setShow: (show) => set({ show }),
  setPlaceholder: (placeholder) => set({ placeholder }),
  setValue: (value) => set({ value }),
  requestFocus: () => set((state) => ({ focusTick: state.focusTick + 1 }))
}));
