import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useEffect, useRef, useState } from 'react';
import type { Plan } from '@/bridge/plan';
import { planComplete } from '@/bridge/plan';
import { errorMessage } from '@/shared/lib/error';
import styles from './complete-dialog.module.css';
import { refreshPlanViews } from './query';

type CompleteDialogProps = {
  plan: Plan | null;
  onClose: () => void;
};

export function CompleteDialog({ plan, onClose }: CompleteDialogProps) {
  if (!plan) {
    return null;
  }
  return <CompletePanel plan={plan} onClose={onClose} />;
}

function CompletePanel({ plan, onClose }: { plan: Plan; onClose: () => void }) {
  const panelRef = useRef<HTMLDivElement>(null);
  const returnFocus = useRef<HTMLElement | null>(null);
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;
  const [result, setResult] = useState('');
  const queryClient = useQueryClient();
  const mutation = useMutation({
    mutationFn: (value: string) => planComplete(plan.id, value),
    onSuccess: async () => {
      await refreshPlanViews(queryClient);
      onCloseRef.current();
    }
  });

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
        {mutation.error ? <p className={styles.error}>{errorMessage(mutation.error)}</p> : null}
        <div className={styles.actions}>
          <button type="button" className={styles.ghost} disabled={mutation.isPending} onClick={onClose}>
            取消
          </button>
          <button
            type="button"
            className={styles.primary}
            disabled={mutation.isPending}
            onClick={() => mutation.mutate(result.trim())}
          >
            完成
          </button>
        </div>
      </div>
    </div>
  );
}
