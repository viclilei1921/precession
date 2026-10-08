import { useEffect, useLayoutEffect, useState } from 'react';
import { isMobile } from '@/bridge';
import { CaptureDialog } from './capture';
import { CommandPalette } from './command';
import { DesktopLayout } from './desktop';
import { MobileLayout } from './mobile';
import { useTitleBarActions } from './titlebar-actions';

export function Layout() {
  const [mobile] = useState(isMobile());
  const [captureOpen, setCaptureOpen] = useState(false);
  const [commandOpen, setCommandOpen] = useState(false);
  const { setActions } = useTitleBarActions();

  useLayoutEffect(() => {
    if (mobile) {
      return;
    }
    setActions({
      onCapture: () => setCaptureOpen(true),
      onCommand: () => setCommandOpen(true)
    });
    return () => setActions(null);
  }, [mobile, setActions]);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        setCommandOpen(true);
      }
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  return (
    <>
      {mobile ? <MobileLayout onCapture={() => setCaptureOpen(true)} /> : <DesktopLayout />}
      <CaptureDialog open={captureOpen} onClose={() => setCaptureOpen(false)} />
      <CommandPalette open={commandOpen} onClose={() => setCommandOpen(false)} />
    </>
  );
}
