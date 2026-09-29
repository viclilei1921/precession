import { useEffect, useState } from 'react';
import { isMobile } from '@/bridge';
import { CaptureDialog } from './capture';
import { CommandPalette } from './command';
import { DesktopLayout } from './desktop';
import { MobileLayout } from './mobile';

export function Layout() {
  const [mobile] = useState(isMobile());
  const [captureOpen, setCaptureOpen] = useState(false);
  const [commandOpen, setCommandOpen] = useState(false);

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
      {mobile ? (
        <MobileLayout onCapture={() => setCaptureOpen(true)} />
      ) : (
        <DesktopLayout onCapture={() => setCaptureOpen(true)} onCommand={() => setCommandOpen(true)} />
      )}
      <CaptureDialog open={captureOpen} onClose={() => setCaptureOpen(false)} />
      <CommandPalette open={commandOpen} onClose={() => setCommandOpen(false)} />
    </>
  );
}
