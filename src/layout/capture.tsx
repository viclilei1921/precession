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
