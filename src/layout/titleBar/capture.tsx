import { PlusIcon } from '@phosphor-icons/react';
import { useCaptureStore } from '@/layout/contentView/capture';
import { useAppStore } from '@/store/app';
import styles from './index.module.css';

/** 桌面标题栏上的记一笔。锁定时不显示。 */
export function TitleBarCapture() {
  const unlocked = useAppStore((state) => state.dbStatus?.unlocked);
  const openCapture = useCaptureStore((state) => state.openCapture);

  if (!unlocked) {
    return null;
  }

  return (
    <button type="button" className={styles.capture} onClick={openCapture}>
      <PlusIcon className={styles.icon} weight="regular" />
      记一笔
    </button>
  );
}
