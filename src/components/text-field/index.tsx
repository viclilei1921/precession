import type { ComponentProps } from 'react';
import { Field } from '@/components/field';
import fieldStyles from '@/components/field/index.module.css';

type TextFieldProps = Omit<ComponentProps<'input'>, 'className'> & {
  label?: string;
  hint?: string;
  error?: string;
  className?: string;
};

/** 单行描边输入。标签从框内浮到左上角缺口。 */
export function TextField({
  label,
  hint,
  error,
  className,
  id,
  required,
  disabled,
  placeholder,
  'aria-describedby': describedBy,
  ...props
}: TextFieldProps) {
  return (
    <Field
      id={id}
      label={label}
      hint={hint}
      error={error}
      required={required}
      disabled={disabled}
      className={className}
    >
      {({ id: fieldId, describedBy: fieldDescribedBy }) => (
        <input
          {...props}
          id={fieldId}
          className={fieldStyles.control}
          required={required}
          disabled={disabled}
          placeholder={placeholder ?? ' '}
          aria-invalid={error ? true : props['aria-invalid']}
          aria-describedby={[fieldDescribedBy, describedBy].filter(Boolean).join(' ') || undefined}
        />
      )}
    </Field>
  );
}
