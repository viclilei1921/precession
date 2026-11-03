import {
  CalendarBlankIcon,
  CaretDoubleLeftIcon,
  CaretDoubleRightIcon,
  CaretLeftIcon,
  CaretRightIcon
} from '@phosphor-icons/react';
import clsx from 'clsx';
import { useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
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
  formatDate,
  formatMonth,
  fromDateInputValue,
  isSameLocalDay,
  startOfLocalDay,
  startOfLocalMonth,
  startOfLocalWeek,
  toDateInputValue,
  weekdayNarrow
} from '@/utils/day';
import styles from './index.module.css';

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

function displayValue(value: string, fallback: string): string {
  const time = fromDateInputValue(value);
  if (time == null) {
    return fallback;
  }
  return formatDate(time);
}

function CalendarPanel({ value, onPick }: { value: string; onPick: (value: string) => void }) {
  const { t } = useTranslation('ui');
  const selected = fromDateInputValue(value);
  const today = startOfLocalDay();
  const [month, setMonth] = useState(() => startOfLocalMonth(selected ?? today));
  const cells = monthCells(month);
  const weekdayStart = startOfLocalWeek(today);
  const weekdays = Array.from({ length: 7 }, (_, index) => {
    const time = addLocalDays(weekdayStart, index);
    return { time, label: weekdayNarrow(time) };
  });
  const heading = t('date.yearMonth', {
    year: new Date(month).getFullYear(),
    month: formatMonth(month)
  });

  return (
    <div className={styles.calendar}>
      <header className={styles.header}>
        <IconButton label={t('date.previousYear')} onClick={() => setMonth(addLocalMonths(month, -12))}>
          <CaretDoubleLeftIcon weight="regular" />
        </IconButton>
        <IconButton label={t('date.previousMonth')} onClick={() => setMonth(addLocalMonths(month, -1))}>
          <CaretLeftIcon weight="regular" />
        </IconButton>
        <span className={styles.heading}>{heading}</span>
        <IconButton label={t('date.nextMonth')} onClick={() => setMonth(addLocalMonths(month, 1))}>
          <CaretRightIcon weight="regular" />
        </IconButton>
        <IconButton label={t('date.nextYear')} onClick={() => setMonth(addLocalMonths(month, 12))}>
          <CaretDoubleRightIcon weight="regular" />
        </IconButton>
      </header>
      <div className={styles.weekdays}>
        {weekdays.map((day) => (
          <span key={day.time}>{day.label}</span>
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
  const { t } = useTranslation('ui');
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
              {displayValue(value, t('date.pickDate'))}
            </RowButton>
            {value ? (
              <Button variant="text" disabled={disabled} onClick={() => onChange('')}>
                {t('common:button.clear')}
              </Button>
            ) : null}
            <IconButton label={t('date.openCalendar')} disabled={disabled} expanded={open} onClick={show}>
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
