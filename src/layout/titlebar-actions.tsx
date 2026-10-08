import type { ReactNode } from 'react';
import { createContext, useContext, useMemo, useState } from 'react';

type TitleBarActions = {
  onCapture: () => void;
  onCommand: () => void;
};

type TitleBarActionsContextValue = {
  actions: TitleBarActions | null;
  setActions: (actions: TitleBarActions | null) => void;
};

const TitleBarActionsContext = createContext<TitleBarActionsContextValue | null>(null);

export function TitleBarActionsProvider({ children }: { children: ReactNode }) {
  const [actions, setActions] = useState<TitleBarActions | null>(null);
  const value = useMemo(() => ({ actions, setActions }), [actions]);
  return <TitleBarActionsContext.Provider value={value}>{children}</TitleBarActionsContext.Provider>;
}

export function useTitleBarActions() {
  const value = useContext(TitleBarActionsContext);
  if (!value) {
    throw new Error('TitleBarActionsProvider 缺失');
  }
  return value;
}
