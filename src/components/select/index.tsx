import { CaretDownIcon } from '@phosphor-icons/react';
import clsx from 'clsx';
import type { ComponentProps, ReactNode } from 'react';
import { Field } from '@/components/field';
import fieldStyles from '@/components/field/index.module.css';
import styles from './index.module.css';

type SelectProps = Omit<ComponentProps<'select'>, 'className'> & {
  label?: string;
  hint?: string;
  error?: string;
  className?: string;
  children: ReactNode;
};

function hasValue(value: ComponentProps<'select'>['value']) {
  if (Array.isArray(value)) {
    return value.length > 0;
  }
  return value != null && String(value).length > 0;
}

/** 原生下拉。空值时标签在框内，选中后浮到缺口。 */
export function Select({
  label,
  hint,
  error,
  className,
  id,
  required,
  disabled,
  value,
  children,
  'aria-describedby': describedBy,
  ...props
}: SelectProps) {
  return (
    <Field
      id={id}
      label={label}
      hint={hint}
      error={error}
      required={required}
      disabled={disabled}
      filled={hasValue(value)}
      className={className}
    >
      {({ id: fieldId, describedBy: fieldDescribedBy }) => (
        <span className={styles.wrap}>
          <select
            {...props}
            id={fieldId}
            className={clsx(fieldStyles.control, fieldStyles.blankable, styles.select)}
            required={required}
            disabled={disabled}
            value={value}
            aria-invalid={error ? true : props['aria-invalid']}
            aria-describedby={[fieldDescribedBy, describedBy].filter(Boolean).join(' ') || undefined}
          >
            {children}
          </select>
          <CaretDownIcon className={styles.caret} weight="regular" />
        </span>
      )}
    </Field>
  );
}
