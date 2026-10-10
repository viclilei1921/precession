import { MagnifyingGlassIcon } from '@phosphor-icons/react';
import { useEffect, useRef } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { useSearchStore } from '@/store/search';
import styles from './index.module.css';

export function TitleBarSearch() {
  const inputRef = useRef<HTMLInputElement>(null);
  const { show, placeholder, value, focusTick, setValue } = useSearchStore(
    useShallow((state) => ({
      show: state.show,
      placeholder: state.placeholder,
      value: state.value,
      focusTick: state.focusTick,
      setValue: state.setValue
    }))
  );

  useEffect(() => {
    if (focusTick > 0) {
      inputRef.current?.focus();
    }
  }, [focusTick]);

  if (!show) {
    return null;
  }

  const label = placeholder || '搜索';

  return (
    <label className={styles.search}>
      <MagnifyingGlassIcon className={styles.icon} weight="regular" />
      <input
        ref={inputRef}
        className={styles.searchInput}
        value={value}
        placeholder={label}
        aria-label={label}
        onChange={(event) => setValue(event.currentTarget.value)}
      />
    </label>
  );
}
