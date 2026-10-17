import clsx from 'clsx';
import type { ReactNode } from 'react';
import { useId } from 'react';
import styles from './index.module.css';

type FieldIds = {
  id: string;
  describedBy?: string;
};

type FieldProps = {
  id?: string;
  label?: string;
  hint?: ReactNode;
  error?: string;
  required?: boolean;
  disabled?: boolean;
  /** 没有 placeholder 的控件（下拉、日期）靠它判断标签是否浮起 */
  filled?: boolean;
  invalid?: boolean;
  className?: string;
  children: (ids: FieldIds) => ReactNode;
};

/** 描边字段。空着时标签在框内，聚焦或有值后缩到左上角缺口里。 */
export function Field({
  id,
  label,
  hint,
  error,
  required,
  disabled,
  filled,
  invalid,
  className,
  children
}: FieldProps) {
  const autoId = useId();
  const fieldId = id ?? autoId;
  const hintId = hint && !error ? `${fieldId}-hint` : undefined;
  const errorId = error ? `${fieldId}-error` : undefined;
  const describedBy = [hintId, errorId].filter(Boolean).join(' ') || undefined;
  const labelText = label ? `${label}${required ? ' *' : ''}` : '';
  const showInvalid = Boolean(error) || invalid;

  return (
    <div
      className={clsx(styles.field, className)}
      data-filled={filled ? 'true' : undefined}
      data-invalid={showInvalid ? 'true' : undefined}
      data-disabled={disabled ? 'true' : undefined}
    >
      <div className={styles.outline}>
        {children({ id: fieldId, describedBy })}
        {labelText ? (
          <label className={styles.floatLabel} htmlFor={fieldId}>
            {labelText}
          </label>
        ) : null}
        <fieldset className={styles.notch} aria-hidden="true">
          <legend className={styles.legend}>{labelText ? <span>{labelText}</span> : null}</legend>
        </fieldset>
      </div>
      {error ? (
        <p id={errorId} className={styles.error}>
          {error}
        </p>
      ) : null}
      {hint && !error ? (
        <p id={hintId} className={styles.hint}>
          {hint}
        </p>
      ) : null}
    </div>
  );
}
