import { useEffect } from 'react';
import { useSearchStore } from '@/store/search';

/** 当前页面打开标题栏搜索，离开时关闭并清空输入 */
export function usePageSearch(placeholder: string) {
  const value = useSearchStore((state) => state.value);

  useEffect(() => {
    const { setShow, setPlaceholder, setValue } = useSearchStore.getState();
    setShow(true);
    setPlaceholder(placeholder);
    setValue('');

    return () => {
      setShow(false);
      setPlaceholder('');
      setValue('');
    };
  }, [placeholder]);

  return value;
}
