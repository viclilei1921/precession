import { useEffect, useRef, useState } from 'react';
import type { Plan } from '@/bridge/plan';
import styles from './complete-dialog.module.css';

type CompleteDialogProps = {
  plan: Plan | null;
  pending: boolean;
  error: string;
  onSubmit: (result: string) => void;
  onClose: () => void;
};

export function CompleteDialog({ plan, pending, error, onSubmit, onClose }: CompleteDialogProps) {
  if (!plan) {
    return null;
  }
  return <CompletePanel plan={plan} pending={pending} error={error} onSubmit={onSubmit} onClose={onClose} />;
}

function CompletePanel({
  plan,
  pending,
  error,
  onSubmit,
  onClose
}: {
  plan: Plan;
  pending: boolean;
  error: string;
  onSubmit: (result: string) => void;
  onClose: () => void;
}) {
  const panelRef = useRef<HTMLDivElement>(null);
  const returnFocus = useRef<HTMLElement | null>(null);
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;
  const [result, setResult] = useState('');

  useEffect(() => {
    returnFocus.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    panelRef.current?.focus();

    function onKey(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        onCloseRef.current();
      }
    }

    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('keydown', onKey);
      returnFocus.current?.focus();
    };
  }, []);

  return (
    <div className={styles.backdrop}>
      <button type="button" className={styles.scrim} aria-label="关闭" onClick={onClose} />
      <div ref={panelRef} className={styles.dialog} role="dialog" aria-modal="true" aria-label="完成计划" tabIndex={-1}>
        <h2 className={styles.title}>完成计划</h2>
        <p className={styles.name}>{plan.title}</p>
        <label className={styles.field}>
          结果
          <textarea value={result} placeholder="可以补一句结果" onChange={(event) => setResult(event.target.value)} />
        </label>
        {error ? <p className={styles.error}>{error}</p> : null}
        <div className={styles.actions}>
          <button type="button" className={styles.ghost} disabled={pending} onClick={onClose}>
            取消
          </button>
          <button type="button" className={styles.primary} disabled={pending} onClick={() => onSubmit(result.trim())}>
            完成
          </button>
        </div>
      </div>
    </div>
  );
}
