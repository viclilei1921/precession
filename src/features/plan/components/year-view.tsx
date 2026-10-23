import clsx from 'clsx';
import type {
  ButtonHTMLAttributes,
  CSSProperties,
  MouseEventHandler,
  MouseEvent as ReactMouseEvent,
  ReactNode
} from 'react';
import { useEffect, useState } from 'react';
import type { Plan } from '@/bridge/plan';
import { rippleHost, useRipple } from '@/components/ripple';
import { Scroll } from '@/components/scroll';
import { addLocalMonths, formatMonthDay, startOfLocalDay, startOfLocalMonth } from '@/utils/day';
import styles from '../calendar.module.css';
import { miniDays, plansOnDay, WEEK_SHORT } from '../layout';

type YearViewProps = {
  yearStart: number;
  today: number;
  plans: Plan[];
  hueOf: (plan: Plan) => number;
  onOpenMonth: (month: number) => void;
  onOpen: (plan: Plan) => void;
};

type Peek = {
  day: number;
  left: number;
  top: number;
  pinned: boolean;
};

/** 一年十二个月。点日期只看名字，点名字才打开详情 */
export function YearView({ yearStart, today, plans, hueOf, onOpenMonth, onOpen }: YearViewProps) {
  const [peek, setPeek] = useState<Peek | null>(null);
  const year = new Date(yearStart).getFullYear();
  const hits = peek ? plansOnDay(plans, peek.day) : [];

  useEffect(() => {
    if (!peek?.pinned) {
      return;
    }
    function close(event: PointerEvent) {
      const target = event.target;
      if (target instanceof Element && target.closest('[data-peek]')) {
        return;
      }
      setPeek(null);
    }
    document.addEventListener('pointerdown', close);
    return () => document.removeEventListener('pointerdown', close);
  }, [peek?.pinned]);

  function show(day: number, anchor: HTMLElement, pinned: boolean) {
    const rect = anchor.getBoundingClientRect();
    let left = rect.right + 8;
    const top = Math.min(rect.top, window.innerHeight - 160);
    if (left > window.innerWidth - 180) {
      left = Math.max(8, rect.left - 180);
    }
    setPeek({ day, left, top, pinned });
  }

  function hover(event: ReactMouseEvent<HTMLButtonElement>, day: number) {
    if (peek?.pinned) {
      return;
    }
    if (plansOnDay(plans, day).length === 0) {
      setPeek(null);
      return;
    }
    show(day, event.currentTarget, false);
  }

  return (
    <Scroll className={styles.fill} axis="vertical">
      <div className={styles.year}>
        {Array.from({ length: 12 }, (_, month) => {
          const monthStart = startOfLocalMonth(addLocalMonths(new Date(year, 0, 1).getTime(), month));
          const days = miniDays(monthStart);
          return (
            <section key={monthStart}>
              <Press className={styles.miniTitle} onClick={() => onOpenMonth(monthStart)}>
                {`${month + 1}月`}
              </Press>
              <div className={styles.miniWd}>
                {WEEK_SHORT.map((label) => (
                  <span key={label}>{label}</span>
                ))}
              </div>
              <div className={styles.miniGrid}>
                {days.map(({ day, outside }) => {
                  const items = plansOnDay(plans, day);
                  const hue = items[0] ? hueOf(items[0]) : undefined;
                  return (
                    <Press
                      key={day}
                      className={styles.miniDay}
                      data-out={outside ? 'true' : undefined}
                      data-today={startOfLocalDay(day) === today ? 'true' : undefined}
                      data-busy={items.length > 0 ? 'true' : undefined}
                      aria-current={startOfLocalDay(day) === today ? 'date' : undefined}
                      style={hue == null ? undefined : ({ '--hue': hue } as CSSProperties)}
                      onMouseEnter={(event) => hover(event, day)}
                      onMouseLeave={() => {
                        if (!peek?.pinned) {
                          setPeek(null);
                        }
                      }}
                      onClick={(event) => show(day, event.currentTarget, true)}
                    >
                      {new Date(day).getDate()}
                    </Press>
                  );
                })}
              </div>
            </section>
          );
        })}
      </div>
      {peek ? (
        <div className={styles.hover} data-peek="true" style={{ left: peek.left, top: peek.top }}>
          <div className={styles.hoverDate}>{`${new Date(peek.day).getFullYear()}年${formatMonthDay(peek.day)}`}</div>
          {hits.length === 0 ? (
            <p className={styles.hoverEmpty}>这一天没有计划</p>
          ) : (
            <ul>
              {hits.map((plan) => (
                <li key={plan.id}>
                  <button
                    type="button"
                    className={styles.peekPlan}
                    data-done={plan.status === 'done' ? 'true' : undefined}
                    style={{ '--hue': hueOf(plan) } as CSSProperties}
                    onClick={() => onOpen(plan)}
                  >
                    <i />
                    <span>{plan.title}</span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      ) : null}
    </Scroll>
  );
}

function Press({
  className,
  children,
  onClick,
  onMouseEnter,
  onMouseLeave,
  ...rest
}: {
  className?: string;
  children: ReactNode;
  onClick?: MouseEventHandler<HTMLButtonElement>;
  onMouseEnter?: MouseEventHandler<HTMLButtonElement>;
  onMouseLeave?: MouseEventHandler<HTMLButtonElement>;
} & Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'className' | 'onClick' | 'onMouseEnter' | 'onMouseLeave'>) {
  const ripple = useRipple();

  return (
    <button
      type="button"
      {...rest}
      className={clsx(rippleHost, className)}
      onClick={onClick}
      onMouseEnter={onMouseEnter}
      onMouseLeave={onMouseLeave}
      onPointerDown={ripple.onPointerDown}
      onKeyDown={ripple.onKeyDown}
    >
      {ripple.nodes}
      {children}
    </button>
  );
}
