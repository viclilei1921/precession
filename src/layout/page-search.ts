import { useEffect } from 'react';
import { useTitleBarStore } from '@/store/titlebar';

export function usePageSearch(placeholder: string) {
  const query = useTitleBarStore((state) => state.query);
  const openSearch = useTitleBarStore((state) => state.openSearch);
  const closeSearch = useTitleBarStore((state) => state.closeSearch);

  useEffect(() => {
    openSearch(placeholder);
    return () => closeSearch();
  }, [placeholder, openSearch, closeSearch]);

  return query;
}
