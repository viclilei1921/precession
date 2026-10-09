import { useEffect, useLayoutEffect, useState } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { isMobile } from '@/bridge';
import { useTitleBarStore } from '@/store/titlebar';
import { CaptureDialog } from './capture';
import { DesktopLayout } from './desktop';
import { MobileLayout } from './mobile';

export function Layout() {
  const [mobile] = useState(isMobile());
  const { captureOpen, setActive, openCapture, closeCapture, requestFocus } = useTitleBarStore(
    useShallow((state) => ({
      captureOpen: state.captureOpen,
      setActive: state.setActive,
      openCapture: state.openCapture,
      closeCapture: state.closeCapture,
      requestFocus: state.requestFocus
    }))
  );

  useLayoutEffect(() => {
    if (mobile) {
      return;
    }
    setActive(true);
    return () => setActive(false);
  }, [mobile, setActive]);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        requestFocus();
      }
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [requestFocus]);

  return (
    <>
      {mobile ? <MobileLayout onCapture={openCapture} /> : <DesktopLayout />}
      <CaptureDialog open={captureOpen} onClose={closeCapture} />
    </>
  );
}
