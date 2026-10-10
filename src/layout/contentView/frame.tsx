import { useEffect } from 'react';
import { useAppStore } from '@/store/app';
import { useSearchStore } from '@/store/search';
import { CaptureDialog, useCaptureStore } from './capture';
import { DesktopContent } from './desktop';
import styles from './index.module.css';
import { MobileContent } from './mobile';

/** 路由根节点：按设备切换桌面或移动端，并挂上记一笔 */
export function ContentFrame() {
  const mobile = useAppStore((state) => state.isMobile());
  const requestFocus = useSearchStore((state) => state.requestFocus);
  const captureOpen = useCaptureStore((state) => state.open);
  const openCapture = useCaptureStore((state) => state.openCapture);
  const closeCapture = useCaptureStore((state) => state.closeCapture);

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

  useEffect(() => () => closeCapture(), [closeCapture]);

  return (
    <div className={styles.frame}>
      {mobile ? <MobileContent onCapture={openCapture} /> : <DesktopContent />}
      <CaptureDialog open={captureOpen} onClose={closeCapture} />
    </div>
  );
}
