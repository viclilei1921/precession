import { XIcon } from '@phosphor-icons/react';
import type { ReactNode } from 'react';
import { useEffect, useRef } from 'react';
import { IconButton } from '@/components/button/icon';
import { ScrimButton } from '@/components/button/scrim';
import styles from './sheet.module.css';

type SheetProps = {
  title: string;
  subtitle?: string;
  onClose: () => void;
  children: ReactNode;
  /** 计划详情这类长表单用更高的面板 */
  tall?: boolean;
  footer?: ReactNode;
};

/** 从底部升起的面板。关闭交给路由，返回键能把它收起来。 */
export function Sheet({ title, subtitle, onClose, children, tall, footer }: SheetProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;

  useEffect(() => {
    panelRef.current?.focus();

    function onKey(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        onCloseRef.current();
      }
    }

    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, []);

  return (
    <div className={styles.backdrop}>
      <ScrimButton onClick={onClose} />
      <div
        ref={panelRef}
        className={styles.panel}
        data-tall={tall ? 'true' : undefined}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        tabIndex={-1}
      >
        <div className={styles.handle} aria-hidden="true" />
        <header className={styles.head}>
          <div>
            <h2>{title}</h2>
            {subtitle ? <p>{subtitle}</p> : null}
          </div>
          <IconButton label="关闭" size="compact" onClick={onClose}>
            <XIcon weight="regular" />
          </IconButton>
        </header>
        <div className={styles.body}>{children}</div>
        {footer ? <footer className={styles.foot}>{footer}</footer> : null}
      </div>
    </div>
  );
}
