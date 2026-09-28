import { CheckCircle, Circle } from '@phosphor-icons/react';
import type { Plan } from '@/bridge/plan';
import styles from './plan-check.module.css';

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
      <button
        type="button"
        className={styles.check}
        aria-label={done ? '已完成' : '完成'}
        disabled={done}
        onClick={() => onComplete(plan)}
      >
        {done ? (
          <CheckCircle className={styles.icon} weight="regular" />
        ) : (
          <Circle className={styles.icon} weight="regular" />
        )}
      </button>
      <button type="button" className={done ? styles.doneTitle : styles.title} onClick={() => onOpen(plan)}>
        {plan.title}
      </button>
      {meta ? <span className={styles.meta}>{meta}</span> : null}
    </div>
  );
}
