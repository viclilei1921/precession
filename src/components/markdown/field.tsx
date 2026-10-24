import type { Ref } from 'react';
import { forwardRef } from 'react';
import { Field } from '@/components/field';

import type { MarkdownEditorHandle } from './editor';
import { MarkdownEditor } from './editor';
import type { MarkdownImageOwner } from './image';

type MarkdownFieldProps = {
  label?: string;
  hint?: string;
  error?: string;
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  required?: boolean;
  placeholder?: string;
  /** 编辑器整体高度，含工具栏。内容在下面单独滚动 */
  height?: string;
  compact?: boolean;
  className?: string;
  /** 已有所属记录时传入。`null` 表示还没保存，插入按钮停用 */
  image?: MarkdownImageOwner | null;
};

export const MarkdownField = forwardRef(function MarkdownField(
  {
    label,
    hint,
    error,
    value,
    onChange,
    disabled,
    required,
    placeholder,
    height,
    compact,
    className,
    image
  }: MarkdownFieldProps,
  ref: Ref<MarkdownEditorHandle>
) {
  return (
    <Field label={label} hint={hint} error={error} required={required} disabled={disabled} filled className={className}>
      {({ id }) => (
        <MarkdownEditor
          ref={ref}
          id={id}
          value={value}
          onChange={onChange}
          disabled={disabled}
          placeholder={placeholder}
          height={height}
          compact={compact}
          image={image}
        />
      )}
    </Field>
  );
});
