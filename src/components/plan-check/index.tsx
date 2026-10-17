import { CheckCircleIcon, CircleIcon } from '@phosphor-icons/react';
import type { Plan } from '@/bridge/plan';
import { IconButton } from '@/components/button/icon';
import { RowButton } from '@/components/button/row';
import styles from './index.module.css';

type PlanCheckProps = {
  plan: Plan;
  meta?: string;
  onComplete: (plan: Plan) => void;
  onOpen: (plan: Plan) => void;
};

export function PlanCheck({ plan, meta, onComplete, onOpen }: PlanCheckProps) {
  const done = plan.status === 'done';

  return (
    <div className={styles.row}>
      <IconButton
        label={done ? '已完成' : '完成'}
        size="plain"
        className={styles.check}
        disabled={done}
        onClick={() => onComplete(plan)}
      >
        {done ? <CheckCircleIcon weight="regular" /> : <CircleIcon weight="regular" />}
      </IconButton>
      <RowButton variant="title" className={done ? styles.doneTitle : undefined} onClick={() => onOpen(plan)}>
        {plan.title}
      </RowButton>
      {meta ? <span className={styles.meta}>{meta}</span> : null}
    </div>
  );
}
