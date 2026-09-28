import type { ReactNode } from 'react';
import { useEffect, useRef } from 'react';
import { isMobilePlatform } from '@/platform';
import styles from './layout.module.css';

type OverlayProps = {
  open: boolean;
  title: string;
  onClose: () => void;
  children: ReactNode;
};

export function Overlay({ open, title, onClose, children }: OverlayProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  const returnFocus = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (!open) {
      return;
    }
    returnFocus.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    panelRef.current?.focus();

    function onKey(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        onClose();
      }
    }

    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('keydown', onKey);
      returnFocus.current?.focus();
    };
  }, [open, onClose]);

  if (!open) {
    return null;
  }

  const placement = isMobilePlatform() ? styles.sheet : styles.dialog;

  return (
    <div className={styles.backdrop}>
      <button type="button" className={styles.scrim} aria-label="关闭" onClick={onClose} />
      <div ref={panelRef} className={placement} role="dialog" aria-modal="true" aria-label={title} tabIndex={-1}>
        <h2 className={styles.overlayTitle}>{title}</h2>
        {children}
      </div>
    </div>
  );
}
