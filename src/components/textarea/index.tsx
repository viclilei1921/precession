import clsx from 'clsx';
import type { ComponentProps } from 'react';
import { Field } from '@/components/field';
import fieldStyles from '@/components/field/index.module.css';

type TextareaProps = Omit<ComponentProps<'textarea'>, 'className'> & {
  label?: string;
  hint?: string;
  error?: string;
  className?: string;
};

/** 多行描边输入。标签动画与 Text Field 相同，可以纵向拖动。 */
export function Textarea({
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
}: TextareaProps) {
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
        <textarea
          {...props}
          id={fieldId}
          className={clsx(fieldStyles.control, fieldStyles.textarea)}
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
