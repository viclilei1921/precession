import { useState } from 'react';
import type { Plan } from '@/bridge/plan';
import { Button } from '@/components/button';
import { MarkdownField } from '@/components/markdown';
import { Modal } from '@/components/modal';
import styles from './index.module.css';

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
  const [result, setResult] = useState('');

  return (
    <Modal title="完成计划" onClose={onClose}>
      <p className={styles.name}>{plan.title}</p>
      <MarkdownField compact label="结果" value={result} placeholder="可以补一句结果" onChange={setResult} />
      {error ? <p className={styles.error}>{error}</p> : null}
      <div className={styles.actions}>
        <Button variant="outlined" disabled={pending} onClick={onClose}>
          取消
        </Button>
        <Button disabled={pending} onClick={() => onSubmit(result.trim())}>
          完成
        </Button>
      </div>
    </Modal>
  );
}
