import type { ReactNode, TransitionEvent } from 'react';
import { createContext, useContext, useEffect, useRef, useState } from 'react';
import { ScrimButton } from '@/components/button/scrim';
import styles from './sheet.module.css';

type Phase = 'closed' | 'open' | 'out';

type SheetProps = {
  title: string;
  subtitle?: string;
  onClose: () => void;
  children: ReactNode;
  /** 计划详情这类长表单用更高的面板 */
  tall?: boolean;
  footer?: ReactNode;
};

const SheetCloseContext = createContext<(() => void) | null>(null);

/** 面板里的按钮用这个关闭，这样会先播收起动画。 */
export function useSheetClose() {
  return useContext(SheetCloseContext);
}

/** 从底部升起的面板。点遮罩关闭，面板上不放关闭图标。 */
export function Sheet({ title, subtitle, onClose, children, tall, footer }: SheetProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  const onCloseRef = useRef(onClose);
  const phaseRef = useRef<Phase>('closed');
  const finished = useRef(false);
  const [phase, setPhase] = useState<Phase>('closed');
  onCloseRef.current = onClose;
  phaseRef.current = phase;

  function finish() {
    if (finished.current) {
      return;
    }
    finished.current = true;
    onCloseRef.current();
  }

  function requestClose() {
    if (phaseRef.current === 'out') {
      return;
    }
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
      finish();
      return;
    }
    setPhase('out');
  }

  const requestCloseRef = useRef(requestClose);
  const finishRef = useRef(finish);
  requestCloseRef.current = requestClose;
  finishRef.current = finish;

  useEffect(() => {
    panelRef.current?.focus({ preventScroll: true });
    // 先画出停在屏幕外的那一帧，再滑上来。挂载时直接播动画会先闪在终点。
    let inner = 0;
    const outer = requestAnimationFrame(() => {
      inner = requestAnimationFrame(() => {
        setPhase((current) => (current === 'closed' ? 'open' : current));
      });
    });

    function onKey(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        requestCloseRef.current();
      }
    }

    document.addEventListener('keydown', onKey);
    return () => {
      cancelAnimationFrame(outer);
      cancelAnimationFrame(inner);
      document.removeEventListener('keydown', onKey);
    };
  }, []);

  useEffect(() => {
    if (phase !== 'out') {
      return;
    }
    const timer = window.setTimeout(() => finishRef.current(), 320);
    return () => window.clearTimeout(timer);
  }, [phase]);

  function onPanelTransitionEnd(event: TransitionEvent<HTMLDivElement>) {
    if (event.target !== event.currentTarget || phase !== 'out' || !event.propertyName.includes('transform')) {
      return;
    }
    finish();
  }

  return (
    <div className={styles.backdrop} data-phase={phase}>
      <div className={styles.shade}>
        <ScrimButton onClick={requestClose} />
      </div>
      <SheetCloseContext.Provider value={requestClose}>
        <div
          ref={panelRef}
          className={styles.panel}
          data-tall={tall ? 'true' : undefined}
          role="dialog"
          aria-modal="true"
          aria-label={title}
          tabIndex={-1}
          onTransitionEnd={onPanelTransitionEnd}
        >
          <div className={styles.handle} aria-hidden="true" />
          <header className={styles.head}>
            <div>
              <h2>{title}</h2>
              {subtitle ? <p>{subtitle}</p> : null}
            </div>
          </header>
          <div className={styles.body}>{children}</div>
          {footer ? <footer className={styles.foot}>{footer}</footer> : null}
        </div>
      </SheetCloseContext.Provider>
    </div>
  );
}
