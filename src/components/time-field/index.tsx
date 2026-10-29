import clsx from 'clsx';
import type { KeyboardEvent as ReactKeyboardEvent } from 'react';
import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { Field } from '@/components/field';
import fieldStyles from '@/components/field/index.module.css';
import { joinTime, secondsOf, splitSeconds } from './format';
import styles from './index.module.css';

type Segment = 'hour' | 'minute' | 'second' | 'millisecond';

type TimeFieldProps = {
  label?: string;
  /** 没有可见标签时读出来 */
  ariaLabel?: string;
  /** 秒，含小数 */
  value: number;
  error?: string;
  /** 只把边框标红，不另起一行说明 */
  invalid?: boolean;
  className?: string;
  onChange: (seconds: number) => void;
};

const range: Record<Segment, [number, number]> = {
  hour: [0, 2],
  minute: [3, 5],
  second: [6, 8],
  millisecond: [9, 12]
};

const nextSegment: Record<Segment, Segment> = {
  hour: 'minute',
  minute: 'second',
  second: 'millisecond',
  millisecond: 'hour'
};

const previousSegment: Record<Segment, Segment> = {
  hour: 'millisecond',
  minute: 'hour',
  second: 'minute',
  millisecond: 'second'
};

type Parts = {
  hour: string;
  minute: string;
  second: string;
  millisecond: string;
  pos: Segment;
  cache: string;
};

/** 时分秒输入。点某一段后用数字键写入，方向键改这段或换段。 */
export function TimeField({ label, ariaLabel, value, error, invalid, className, onChange }: TimeFieldProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const parts = useRef<Parts>({ ...splitSeconds(value), pos: 'second', cache: '' });
  const selection = useRef<[number, number] | null>(null);
  const focused = useRef(false);
  const emitted = useRef(secondsOf(parts.current));
  const onChangeRef = useRef(onChange);
  const [text, setText] = useState(() => joinTime(parts.current));
  onChangeRef.current = onChange;

  useEffect(() => {
    if (focused.current) {
      return;
    }
    parts.current = { ...parts.current, ...splitSeconds(value), cache: '' };
    emitted.current = value;
    setText(joinTime(parts.current));
  }, [value]);

  useLayoutEffect(() => {
    const input = inputRef.current;
    const next = selection.current;
    if (!input || !next || document.activeElement !== input || input.value !== text) {
      return;
    }
    input.setSelectionRange(next[0], next[1]);
  }, [text]);

  function paint(start?: number, end?: number) {
    const next = joinTime(parts.current);
    if (start !== undefined && end !== undefined) {
      selection.current = [start, end];
      inputRef.current?.setSelectionRange(start, end);
    }
    setText(next);
    const seconds = secondsOf(parts.current);
    if (seconds !== emitted.current) {
      emitted.current = seconds;
      onChangeRef.current(seconds);
    }
  }

  function selectSegment(segment: Segment) {
    parts.current.pos = segment;
    parts.current.cache = '';
    const [start, end] = range[segment];
    selection.current = [start, end];
    inputRef.current?.focus();
    paint(start, end);
  }

  function segmentAt(end: number | null): Segment | null {
    if (end === null) {
      return null;
    }
    if (end < 3) {
      return 'hour';
    }
    if (end < 6) {
      return 'minute';
    }
    if (end < 9) {
      return 'second';
    }
    if (end <= 12) {
      return 'millisecond';
    }
    return null;
  }

  function writeDigit(digit: string) {
    const current = parts.current;
    if (current.pos === 'millisecond') {
      if (current.cache.length >= 3) {
        current.cache = '';
      }
      current.cache += digit;
      current.millisecond = current.cache.padStart(3, '0');
      paint(9, 12);
      return;
    }

    const reset = current.cache.length >= 2 || current.cache === '0' || Number(current.cache) > 5;
    if (reset) {
      current.cache = '';
    }
    current.cache += digit;
    const number = Number(current.cache);
    const shown = String(number).padStart(2, '0');
    if (current.pos === 'hour') {
      current.hour = shown;
      if (number >= 10) {
        selectSegment('minute');
        return;
      }
      paint(0, 2);
      return;
    }
    if (current.pos === 'minute') {
      current.minute = shown;
      if (number >= 10) {
        selectSegment('second');
        return;
      }
      paint(3, 5);
      return;
    }
    current.second = shown;
    if (number >= 10) {
      selectSegment('millisecond');
      return;
    }
    paint(6, 8);
  }

  function step(direction: number) {
    const current = parts.current;
    const number = Number(current[current.pos]) + direction;
    if (current.pos === 'hour') {
      if (number < 0 || number > 23) {
        return;
      }
      current.hour = String(number).padStart(2, '0');
      paint(0, 2);
      return;
    }
    if (current.pos === 'minute') {
      if (number < 0 || number > 59) {
        return;
      }
      current.minute = String(number).padStart(2, '0');
      paint(3, 5);
      return;
    }
    if (current.pos === 'second') {
      if (number < 0 || number > 59) {
        return;
      }
      current.second = String(number).padStart(2, '0');
      paint(6, 8);
      return;
    }
    if (number < 0 || number > 999) {
      return;
    }
    current.millisecond = String(number).padStart(3, '0');
    paint(9, 12);
  }

  function clearSegment() {
    const current = parts.current;
    current.cache = '';
    if (current.pos === 'millisecond') {
      current.millisecond = '000';
      paint(9, 12);
      return;
    }
    if (current.pos === 'second') {
      current.second = '00';
      paint(6, 8);
      return;
    }
    if (current.pos === 'minute') {
      current.minute = '00';
      paint(3, 5);
      return;
    }
    current.hour = '00';
    paint(0, 2);
  }

  function onKeyDown(event: ReactKeyboardEvent<HTMLInputElement>) {
    const { key, code } = event;
    if (key === 'Tab') {
      return;
    }
    event.preventDefault();
    if (key === 'Unidentified' || text.length !== 12) {
      paint();
      return;
    }
    if (code.startsWith('Digit') || (code.startsWith('Numpad') && code !== 'NumpadDecimal')) {
      const digit = key.slice(-1);
      if (/^\d$/.test(digit)) {
        writeDigit(digit);
      }
      return;
    }
    if (key === 'ArrowUp') {
      step(1);
      return;
    }
    if (key === 'ArrowDown') {
      step(-1);
      return;
    }
    if (key === 'ArrowLeft') {
      selectSegment(previousSegment[parts.current.pos]);
      return;
    }
    if (key === 'ArrowRight') {
      selectSegment(nextSegment[parts.current.pos]);
      return;
    }
    if (key === 'Backspace' || key === 'Delete') {
      clearSegment();
    }
  }

  return (
    <Field label={label} error={error} invalid={invalid} filled className={className}>
      {({ id, describedBy }) => (
        <input
          ref={inputRef}
          id={id}
          className={clsx(fieldStyles.control, styles.input)}
          type="text"
          inputMode="numeric"
          autoComplete="off"
          value={text}
          aria-label={label ? undefined : ariaLabel}
          aria-invalid={error || invalid ? true : undefined}
          aria-describedby={describedBy}
          onFocus={() => {
            focused.current = true;
            const [start, end] = range[parts.current.pos];
            selection.current = [start, end];
            inputRef.current?.setSelectionRange(start, end);
          }}
          onBlur={() => {
            focused.current = false;
            parts.current.cache = '';
            paint();
          }}
          onPointerDown={() => {
            const finish = () => {
              document.removeEventListener('pointerup', finish);
              const segment = segmentAt(inputRef.current?.selectionEnd ?? null);
              if (segment) {
                selectSegment(segment);
              }
            };
            document.addEventListener('pointerup', finish);
          }}
          onKeyDown={onKeyDown}
          onClick={(event) => event.preventDefault()}
          onDoubleClick={(event) => event.preventDefault()}
          onPointerMove={(event) => event.preventDefault()}
        />
      )}
    </Field>
  );
}
