import type { ReactNode } from 'react';
import { useEffect, useRef } from 'react';
import styles from './index.module.css';

type ModalProps = {
  title: string;
  onClose: () => void;
  children: ReactNode;
  wide?: boolean;
};

/** 计划用的模态框。桌面居中，移动端贴底。 */
export function Modal({ title, onClose, children, wide }: ModalProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  const returnFocus = useRef<HTMLElement | null>(null);
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;

  useEffect(() => {
    returnFocus.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    panelRef.current?.focus();

    function onKey(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        onCloseRef.current();
      }
    }

    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('keydown', onKey);
      returnFocus.current?.focus();
    };
  }, []);

  return (
    <div className={styles.backdrop}>
      <button type="button" className={styles.scrim} aria-label="关闭" onClick={onClose} />
      <div
        ref={panelRef}
        className={wide ? styles.wide : styles.dialog}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        tabIndex={-1}
      >
        <div className={styles.panel}>
          <h2 className={styles.title}>{title}</h2>
          {children}
        </div>
      </div>
    </div>
  );
}
