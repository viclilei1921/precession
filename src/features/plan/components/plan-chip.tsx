import clsx from 'clsx';
import type { CSSProperties, MouseEvent } from 'react';
import type { Plan } from '@/bridge/plan';
import styles from '../calendar.module.css';

type PlanChipProps = {
  plan: Plan;
  hue: number;
  time?: string;
  /** 标题下面的一行，例如跨天的起止日期 */
  detail?: string;
  className?: string;
  style?: CSSProperties;
  before?: boolean;
  after?: boolean;
  onOpen: (plan: Plan) => void;
  onToggle: (plan: Plan) => void;
};

/** 日历上的一条计划。勾选只改完成，点标题打开详情 */
export function PlanChip({
  plan,
  hue,
  time,
  detail,
  className,
  style,
  before,
  after,
  onOpen,
  onToggle
}: PlanChipProps) {
  const done = plan.status === 'done';

  function toggle(event: MouseEvent<HTMLButtonElement>) {
    event.stopPropagation();
    onToggle(plan);
  }

  return (
    <div
      className={clsx(styles.chip, className)}
      style={{ ...style, '--hue': hue } as CSSProperties}
      data-chip="true"
      data-done={done ? 'true' : undefined}
      data-before={before ? 'true' : undefined}
      data-after={after ? 'true' : undefined}
    >
      <button
        type="button"
        className={styles.box}
        data-check="true"
        data-on={done ? 'true' : undefined}
        aria-pressed={done}
        aria-label={done ? '标为未完成' : '标为完成'}
        onPointerDown={(event) => event.stopPropagation()}
        onClick={toggle}
      >
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <path d="M3.5 8.2 6.6 11.3 12.5 4.7" />
        </svg>
      </button>
      <button type="button" className={styles.chipMain} onClick={() => onOpen(plan)}>
        <span className={styles.chipText}>
          <span className={styles.chipTitle}>{plan.title}</span>
          {detail ? <span className={styles.chipRange}>{detail}</span> : null}
        </span>
        {time ? <span className={styles.chipTime}>{time}</span> : null}
      </button>
    </div>
  );
}
