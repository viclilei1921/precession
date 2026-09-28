import { useNavigate } from '@tanstack/react-router';
import { useState } from 'react';
import { routeItems } from '@/router/items';
import styles from './layout.module.css';
import { Overlay } from './overlay';

type CommandPaletteProps = {
  open: boolean;
  onClose: () => void;
};

export function CommandPalette({ open, onClose }: CommandPaletteProps) {
  const navigate = useNavigate();
  const [query, setQuery] = useState('');
  const keyword = query.trim();
  const jumps = routeItems.filter((item) => keyword.length === 0 || item.label.includes(keyword));

  return (
    <Overlay open={open} title="跳转" onClose={onClose}>
      <input
        className={styles.commandInput}
        value={query}
        placeholder="跳转或搜索"
        aria-label="跳转或搜索"
        onChange={(event) => setQuery(event.currentTarget.value)}
      />
      <ul className={styles.choiceList}>
        {jumps.map((item) => (
          <li key={item.id}>
            <button
              type="button"
              className={styles.choice}
              onClick={() => {
                onClose();
                setQuery('');
                void navigate({ to: item.to });
              }}
            >
              {item.label}
            </button>
          </li>
        ))}
      </ul>
      <p className={styles.emptySearch}>记录搜索还没有内容。</p>
    </Overlay>
  );
}
