import { useEffect, useLayoutEffect, useState } from 'react';
import { isMobile } from '@/bridge';
import { useTitleBarActions } from '@/store/titlebar';
import { CaptureDialog } from './capture';
import { DesktopLayout } from './desktop';
import { MobileLayout } from './mobile';

export function Layout() {
  const [mobile] = useState(isMobile());
  const captureOpen = useTitleBarActions((state) => state.captureOpen);
  const setActive = useTitleBarActions((state) => state.setActive);
  const openCapture = useTitleBarActions((state) => state.openCapture);
  const closeCapture = useTitleBarActions((state) => state.closeCapture);
  const requestFocus = useTitleBarActions((state) => state.requestFocus);

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
