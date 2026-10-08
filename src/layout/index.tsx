import { useEffect, useLayoutEffect, useState } from 'react';
import { isMobile } from '@/bridge';
import { useTitleBarActions } from '@/store/titlebar';
import { CaptureDialog } from './capture';
import { CommandPalette } from './command';
import { DesktopLayout } from './desktop';
import { MobileLayout } from './mobile';

export function Layout() {
  const [mobile] = useState(isMobile());
  const captureOpen = useTitleBarActions((state) => state.captureOpen);
  const commandOpen = useTitleBarActions((state) => state.commandOpen);
  const setActive = useTitleBarActions((state) => state.setActive);
  const openCapture = useTitleBarActions((state) => state.openCapture);
  const closeCapture = useTitleBarActions((state) => state.closeCapture);
  const openCommand = useTitleBarActions((state) => state.openCommand);
  const closeCommand = useTitleBarActions((state) => state.closeCommand);

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
        openCommand();
      }
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [openCommand]);

  return (
    <>
      {mobile ? <MobileLayout onCapture={openCapture} /> : <DesktopLayout />}
      <CaptureDialog open={captureOpen} onClose={closeCapture} />
      <CommandPalette open={commandOpen} onClose={closeCommand} />
    </>
  );
}
