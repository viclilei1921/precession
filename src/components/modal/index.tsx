import type { MutableRefObject, ReactNode, TransitionEvent } from 'react';
import { createContext, useContext, useEffect, useRef, useState } from 'react';
import { ScrimButton } from '@/components/button/scrim';
import { Scroll } from '@/components/scroll';
import { useAppStore } from '@/store/app';
import styles from './index.module.css';

type Phase = 'closed' | 'open' | 'out';

type ModalProps = {
  title: string;
  onClose: () => void;
  children: ReactNode;
  wide?: boolean;
};

const ModalCloseContext = createContext<(() => void) | null>(null);

/** 面板里的按钮用这个关闭。移动端会先播收起动画。 */
export function useModalClose() {
  return useContext(ModalCloseContext);
}

/** 把带动画的关闭放进 ref，给保存和取消用。 */
export function ModalCloseRef({ target }: { target: MutableRefObject<() => void> }) {
  const requestClose = useModalClose();
  if (requestClose) {
    target.current = requestClose;
  }
  return null;
}

/** 桌面居中。移动端贴底，打开和收起都从底部滑动。 */
export function Modal({ title, onClose, children, wide }: ModalProps) {
  const mobile = useAppStore((state) => state.platform === 'android' || state.platform === 'ios');
  const panelRef = useRef<HTMLDivElement>(null);
  const returnFocus = useRef<HTMLElement | null>(null);
  const onCloseRef = useRef(onClose);
  const mobileRef = useRef(mobile);
  const phaseRef = useRef<Phase>('closed');
  const finished = useRef(false);
  const [phase, setPhase] = useState<Phase>('closed');
  onCloseRef.current = onClose;
  mobileRef.current = mobile;
  phaseRef.current = phase;

  function finish() {
    if (finished.current) {
      return;
    }
    finished.current = true;
    onCloseRef.current();
  }

  function requestClose() {
    if (!mobileRef.current || window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
      finish();
      return;
    }
    if (phaseRef.current === 'out') {
      return;
    }
    setPhase('out');
  }

  const requestCloseRef = useRef(requestClose);
  const finishRef = useRef(finish);
  requestCloseRef.current = requestClose;
  finishRef.current = finish;

  useEffect(() => {
    returnFocus.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    panelRef.current?.focus({ preventScroll: true });
    let inner = 0;
    const outer = requestAnimationFrame(() => {
      inner = requestAnimationFrame(() => {
        if (!mobileRef.current) {
          return;
        }
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
      returnFocus.current?.focus();
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
    <div className={styles.backdrop} data-phase={mobile ? phase : undefined}>
      {mobile ? <div className={styles.shade} /> : null}
      <ScrimButton onClick={requestClose} />
      <ModalCloseContext.Provider value={requestClose}>
        <div
          ref={panelRef}
          className={wide ? styles.wide : styles.dialog}
          role="dialog"
          aria-modal="true"
          aria-label={title}
          tabIndex={-1}
          onTransitionEnd={onPanelTransitionEnd}
        >
          <Scroll className={styles.scroll}>
            <div className={styles.panel}>
              <h2 className={styles.title}>{title}</h2>
              {children}
            </div>
          </Scroll>
        </div>
      </ModalCloseContext.Provider>
    </div>
  );
}
