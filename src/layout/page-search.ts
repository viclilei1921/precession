import { useEffect } from 'react';
import { useTitleBarActions } from '@/store/titlebar';

export function usePageSearch(placeholder: string) {
  const query = useTitleBarActions((state) => state.query);
  const registerSearch = useTitleBarActions((state) => state.registerSearch);

  useEffect(() => registerSearch(placeholder), [placeholder, registerSearch]);

  return query;
}
