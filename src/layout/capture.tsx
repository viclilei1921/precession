import { useNavigate } from '@tanstack/react-router';
import { PATH } from '@/router/path';
import styles from './layout.module.css';
import { Overlay } from './overlay';

const captureTypes = [
  { label: '日记', to: PATH.journal },
  { label: '灵感', to: PATH.journal },
  { label: '写作', to: PATH.journal },
  { label: '计划', to: PATH.plan },
  { label: '里程碑', to: PATH.growth },
  { label: '瞬间', to: PATH.growth },
  { label: '书摘', to: PATH.library }
] as const;

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
                void navigate({ to: item.to });
              }}
            >
              {item.label}
            </button>
          </li>
        ))}
      </ul>
    </Overlay>
  );
}
