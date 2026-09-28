import { useEffect, useState } from 'react';
import { isMobilePlatform } from '@/platform';
import { CaptureDialog } from './capture';
import { CommandPalette } from './command';
import { DesktopLayout } from './desktop';
import { MobileLayout } from './mobile';

export function Layout() {
  const [mobile, setMobile] = useState(isMobilePlatform);
  const [captureOpen, setCaptureOpen] = useState(false);
  const [commandOpen, setCommandOpen] = useState(false);

  useEffect(() => {
    const root = document.documentElement;
    const observer = new MutationObserver(() => setMobile(isMobilePlatform()));
    observer.observe(root, { attributes: true, attributeFilter: ['data-platform'] });
    return () => observer.disconnect();
  }, []);

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
