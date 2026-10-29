import clsx from 'clsx';
import type { ComponentProps, PointerEvent } from 'react';
import { useId } from 'react';
import fieldStyles from '@/components/field/index.module.css';
import { rippleHost, useRipple } from '@/components/ripple';
import styles from './index.module.css';

type CheckboxProps = Omit<ComponentProps<'input'>, 'type' | 'className'> & {
  label?: string;
  error?: string;
  className?: string;
};

/** 勾选框。点击有圆形水波纹，勾选标记缩放入场。 */
export function Checkbox({
  label,
  error,
  className,
  id,
  disabled,
  'aria-describedby': describedBy,
  ...props
}: CheckboxProps) {
  const autoId = useId();
  const fieldId = id ?? autoId;
  const errorId = error ? `${fieldId}-error` : undefined;
  const ripple = useRipple();

  function press(event: PointerEvent<HTMLSpanElement>) {
    if (disabled) {
      return;
    }
    ripple.onPointerDown(event);
  }

  return (
    <div className={clsx(styles.stack, className)}>
      <label className={styles.root} htmlFor={fieldId}>
        <input
          {...props}
          id={fieldId}
          className={styles.input}
          type="checkbox"
          disabled={disabled}
          aria-invalid={error ? true : props['aria-invalid']}
          aria-describedby={[errorId, describedBy].filter(Boolean).join(' ') || undefined}
        />
        <span className={clsx(rippleHost, styles.hit)} onPointerDown={press}>
          {ripple.nodes}
          <span className={styles.box} />
        </span>
        {label ? <span className={styles.label}>{label}</span> : null}
      </label>
      {error ? (
        <p id={errorId} className={fieldStyles.error}>
          {error}
        </p>
      ) : null}
    </div>
  );
}
