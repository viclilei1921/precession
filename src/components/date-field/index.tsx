import { CalendarBlankIcon, CaretLeftIcon, CaretRightIcon } from '@phosphor-icons/react';
import { useEffect, useRef, useState } from 'react';
import { Modal } from '@/components/modal';
import { useAppStore } from '@/store/app';
import {
  addLocalDays,
  addLocalMonths,
  formatMonth,
  fromDateInputValue,
  isSameLocalDay,
  startOfLocalDay,
  startOfLocalMonth,
  toDateInputValue
} from '@/utils/day';
import styles from './index.module.css';

const WEEKDAYS = ['一', '二', '三', '四', '五', '六', '日'];

type DateFieldProps = {
  label: string;
  /** `YYYY-MM-DD`，空字符串表示未选 */
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
};

function monthCells(month: number) {
  const first = new Date(month);
  const offset = (first.getDay() + 6) % 7;
  const from = addLocalDays(month, -offset);
  return Array.from({ length: 42 }, (_, index) => {
    const time = startOfLocalDay(addLocalDays(from, index));
    return { time, inMonth: new Date(time).getMonth() === first.getMonth() };
  });
}

function displayValue(value: string): string {
  const time = fromDateInputValue(value);
  if (time == null) {
    return '选择日期';
  }
  const date = new Date(time);
  return `${date.getFullYear()}年${date.getMonth() + 1}月${date.getDate()}日`;
}

function CalendarPanel({ value, onPick }: { value: string; onPick: (value: string) => void }) {
  const selected = fromDateInputValue(value);
  const today = startOfLocalDay();
  const [month, setMonth] = useState(() => startOfLocalMonth(selected ?? today));
  const cells = monthCells(month);
  const heading = `${new Date(month).getFullYear()}年${formatMonth(month)}`;

  return (
    <div className={styles.calendar}>
      <header className={styles.header}>
        <button
          type="button"
          className={styles.step}
          aria-label="上个月"
          onClick={() => setMonth(addLocalMonths(month, -1))}
        >
          <CaretLeftIcon className={styles.icon} weight="regular" />
        </button>
        <span>{heading}</span>
        <button
          type="button"
          className={styles.step}
          aria-label="下个月"
          onClick={() => setMonth(addLocalMonths(month, 1))}
        >
          <CaretRightIcon className={styles.icon} weight="regular" />
        </button>
      </header>
      <div className={styles.weekdays}>
        {WEEKDAYS.map((day) => (
          <span key={day}>{day}</span>
        ))}
      </div>
      <div className={styles.days}>
        {cells.map((cell) => {
          const picked = selected != null && isSameLocalDay(cell.time, selected);
          return (
            <button
              key={cell.time}
              type="button"
              className={styles.cell}
              data-outside-month={cell.inMonth ? undefined : 'true'}
              data-selected={picked ? 'true' : undefined}
              data-today={isSameLocalDay(cell.time, today) ? 'true' : undefined}
              onClick={() => onPick(toDateInputValue(cell.time))}
            >
              {new Date(cell.time).getDate()}
            </button>
          );
        })}
      </div>
    </div>
  );
}

/** 日期选择。桌面是弹出日历，移动端是底部抽屉。 */
export function DateField({ label, value, onChange, disabled }: DateFieldProps) {
  const mobile = useAppStore((state) => state.isMobile());
  const [open, setOpen] = useState(false);
  const fieldRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open || mobile) {
      return;
    }

    function onPointer(event: PointerEvent) {
      if (!fieldRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    }

    function onKey(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        setOpen(false);
      }
    }

    document.addEventListener('pointerdown', onPointer);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('pointerdown', onPointer);
      document.removeEventListener('keydown', onKey);
    };
  }, [mobile, open]);

  function pick(next: string) {
    onChange(next);
    setOpen(false);
  }

  return (
    <div ref={fieldRef} className={styles.field}>
      <span className={styles.label}>{label}</span>
      <div className={styles.group} data-disabled={disabled ? 'true' : undefined}>
        <button
          type="button"
          className={styles.trigger}
          disabled={disabled}
          onClick={() => setOpen((current) => !current)}
        >
          {displayValue(value)}
        </button>
        {value ? (
          <button type="button" className={styles.textButton} disabled={disabled} onClick={() => onChange('')}>
            清除
          </button>
        ) : null}
        <button
          type="button"
          className={styles.iconButton}
          aria-label="打开日历"
          disabled={disabled}
          onClick={() => setOpen(true)}
        >
          <CalendarBlankIcon className={styles.icon} weight="regular" />
        </button>
      </div>
      {open && !mobile ? (
        <div className={styles.popover}>
          <CalendarPanel value={value} onPick={pick} />
        </div>
      ) : null}
      {open && mobile ? (
        <Modal title={label} onClose={() => setOpen(false)}>
          <CalendarPanel value={value} onPick={pick} />
        </Modal>
      ) : null}
    </div>
  );
}
