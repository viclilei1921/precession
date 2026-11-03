import { useState } from 'react';
import { useTranslation } from 'react-i18next';
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
  const { t } = useTranslation('ui');

  return (
    <Modal title={t('complete.title')} onClose={onClose}>
      <p className={styles.name}>{plan.title}</p>
      <MarkdownField
        compact
        label={t('complete.result')}
        value={result}
        placeholder={t('complete.resultPlaceholder')}
        onChange={setResult}
      />
      {error ? <p className={styles.error}>{error}</p> : null}
      <div className={styles.actions}>
        <Button variant="outlined" disabled={pending} onClick={onClose}>
          {t('common:button.cancel')}
        </Button>
        <Button disabled={pending} onClick={() => onSubmit(result.trim())}>
          {t('complete.submit')}
        </Button>
      </div>
    </Modal>
  );
}
