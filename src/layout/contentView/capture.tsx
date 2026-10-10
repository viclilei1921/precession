import type { Icon } from '@phosphor-icons/react';
import {
  CameraIcon,
  FlagIcon,
  LightningIcon,
  ListChecksIcon,
  NotebookIcon,
  PencilLineIcon,
  QuotesIcon
} from '@phosphor-icons/react';
import { useNavigate } from '@tanstack/react-router';
import { create } from 'zustand';
import { PATH } from '@/router/path';
import styles from './index.module.css';
import { Overlay } from './overlay';

type CaptureState = {
  open: boolean;
  openCapture: () => void;
  closeCapture: () => void;
};

/** 记一笔浮层开关。标题栏和移动端底栏共用。 */
export const useCaptureStore = create<CaptureState>((set) => ({
  open: false,
  openCapture: () => set({ open: true }),
  closeCapture: () => set({ open: false })
}));

const captureTypes: { label: string; Icon: Icon }[] = [
  { label: '日记', Icon: NotebookIcon },
  { label: '灵感', Icon: LightningIcon },
  { label: '写作', Icon: PencilLineIcon },
  { label: '计划', Icon: ListChecksIcon },
  { label: '里程碑', Icon: FlagIcon },
  { label: '瞬间', Icon: CameraIcon },
  { label: '书摘', Icon: QuotesIcon }
];

type CaptureDialogProps = {
  open: boolean;
  onClose: () => void;
};

export function CaptureDialog({ open, onClose }: CaptureDialogProps) {
  const navigate = useNavigate();

  return (
    <Overlay open={open} title="记一笔" onClose={onClose}>
      <ul className={styles.choiceList}>
        {captureTypes.map((item) => (
          <li key={item.label}>
            <button
              type="button"
              className={styles.choice}
              onClick={() => {
                onClose();
                if (item.label === '计划') {
                  void navigate({ to: PATH.plan, search: { create: true } });
                  return;
                }
                if (item.label === '日记' || item.label === '灵感' || item.label === '写作') {
                  const kind = item.label === '日记' ? 'diary' : item.label === '灵感' ? 'spark' : 'writing';
                  void navigate({ to: PATH.journal, search: { create: true, kind } });
                  return;
                }
                if (item.label === '里程碑' || item.label === '瞬间') {
                  void navigate({
                    to: PATH.growth,
                    search: { create: true, kind: item.label === '里程碑' ? 'milestone' : 'moment' }
                  });
                  return;
                }
                void navigate({ to: PATH.library });
              }}
            >
              <item.Icon className={styles.icon} weight="regular" />
              {item.label}
            </button>
          </li>
        ))}
      </ul>
    </Overlay>
  );
}
