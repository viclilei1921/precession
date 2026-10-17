import { CalendarBlankIcon, CaretLeftIcon, CaretRightIcon } from '@phosphor-icons/react';
import clsx from 'clsx';
import { useRef, useState } from 'react';
import { Button } from '@/components/button';
import { DayButton } from '@/components/button/day';
import { IconButton } from '@/components/button/icon';
import { RowButton } from '@/components/button/row';
import { Field } from '@/components/field';
import fieldStyles from '@/components/field/index.module.css';
import { Modal } from '@/components/modal';
import { Popover } from '@/components/popover';
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
  error?: string;
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
        <IconButton label="上个月" onClick={() => setMonth(addLocalMonths(month, -1))}>
          <CaretLeftIcon weight="regular" />
        </IconButton>
        <span>{heading}</span>
        <IconButton label="下个月" onClick={() => setMonth(addLocalMonths(month, 1))}>
          <CaretRightIcon weight="regular" />
        </IconButton>
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
            <DayButton
              key={cell.time}
              selected={picked}
              today={isSameLocalDay(cell.time, today)}
              outside={!cell.inMonth}
              onClick={() => onPick(toDateInputValue(cell.time))}
            >
              {new Date(cell.time).getDate()}
            </DayButton>
          );
        })}
      </div>
    </div>
  );
}

/** 日期选择。桌面用 Popover 浮出日历，移动端是底部抽屉。 */
export function DateField({ label, value, onChange, disabled, error }: DateFieldProps) {
  const mobile = useAppStore((state) => state.isMobile());
  const anchorRef = useRef<HTMLDivElement>(null);
  const [anchorEl, setAnchorEl] = useState<HTMLElement | null>(null);
  const open = anchorEl != null;

  function show() {
    setAnchorEl(anchorRef.current);
  }

  function close() {
    setAnchorEl(null);
  }

  function pick(next: string) {
    onChange(next);
    close();
  }

  return (
    <div ref={anchorRef} className={styles.anchor}>
      <Field label={label} filled={value.length > 0} disabled={disabled} error={error}>
        {() => (
          <div className={styles.group}>
            <RowButton
              className={clsx(fieldStyles.blankable, styles.value)}
              variant="trigger"
              disabled={disabled}
              onClick={show}
            >
              {displayValue(value)}
            </RowButton>
            {value ? (
              <Button variant="text" disabled={disabled} onClick={() => onChange('')}>
                清除
              </Button>
            ) : null}
            <IconButton label="打开日历" disabled={disabled} expanded={open} onClick={show}>
              <CalendarBlankIcon weight="regular" />
            </IconButton>
          </div>
        )}
      </Field>
      {mobile ? (
        open ? (
          <Modal title={label} onClose={close}>
            <CalendarPanel value={value} onPick={pick} />
          </Modal>
        ) : null
      ) : (
        <Popover
          open={open}
          anchorEl={anchorEl}
          label={label}
          className={styles.panel}
          anchorOrigin={{ horizontal: 'left', vertical: 'bottom' }}
          transformOrigin={{ horizontal: 'left', vertical: 'top' }}
          gap={8}
          onClose={close}
        >
          <CalendarPanel value={value} onPick={pick} />
        </Popover>
      )}
    </div>
  );
}
