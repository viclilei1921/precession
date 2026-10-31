import { useEffect, useState } from 'react';
import { create } from 'zustand';
import styles from './toast.module.css';

type ToastState = {
  message: string;
  token: number;
  show: (message: string) => void;
};

/** 移动壳上的短提示。token 每次加一，同一句文案也能再亮一次。 */
export const useToastStore = create<ToastState>((set) => ({
  message: '',
  token: 0,
  show: (message) => set((state) => ({ message, token: state.token + 1 }))
}));

const VISIBLE_MS = 2400;

/** 壳层短提示。挂在布局上，页面用 useToastStore 打开。 */
export function Toast() {
  const message = useToastStore((state) => state.message);
  const token = useToastStore((state) => state.token);
  const [open, setOpen] = useState(false);

  useEffect(() => {
    if (!token || !message) {
      return;
    }
    setOpen(true);
    const timer = window.setTimeout(() => setOpen(false), VISIBLE_MS);
    return () => window.clearTimeout(timer);
  }, [token, message]);

  if (!open || !message) {
    return null;
  }

  return (
    <div className={styles.toast} role="status">
      {message}
    </div>
  );
}
