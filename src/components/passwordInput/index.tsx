import { EyeIcon, EyeSlashIcon } from '@phosphor-icons/react';
import clsx from 'clsx';
import type { ChangeEvent, ComponentProps, InputEvent as ReactInputEvent } from 'react';
import { useState } from 'react';
import styles from './index.module.css';

/** 去掉空格、中文和其他非密码字符。保留英文大小写、数字，以及 ! 到 ~ 的符号。 */
function keepPasswordText(value: string) {
  return value.replace(/[^\x21-\x7E]/g, '');
}

/** 密码输入框 Props */
type PasswordInputProps = Omit<ComponentProps<'input'>, 'type' | 'className'> & {
  /** 外层类名，用来覆盖宽度和间距 */
  className?: string;
  /** 输入框类名，用来覆盖背景和内边距 */
  inputClassName?: string;
};

/** 带小眼睛的密码输入框 */
export function PasswordInput({
  className,
  inputClassName,
  disabled,
  onChange,
  onBeforeInput,
  ...props
}: PasswordInputProps) {
  const [visible, setVisible] = useState(false);

  /** 单字输入直接拦下；粘贴仍交给 onChange 去掉不合规的部分 */
  function beforeInput(event: ReactInputEvent<HTMLInputElement>) {
    const native = event.nativeEvent;
    const blocked =
      native instanceof InputEvent &&
      native.inputType === 'insertText' &&
      native.data != null &&
      keepPasswordText(native.data) !== native.data;
    if (blocked) {
      event.preventDefault();
    }
    onBeforeInput?.(event);
  }

  /** 写入前去掉不合规字符，光标停在被删字符原来的位置 */
  function change(event: ChangeEvent<HTMLInputElement>) {
    const input = event.currentTarget;
    const raw = input.value;
    const next = keepPasswordText(raw);
    if (next !== raw) {
      const end = input.selectionEnd ?? raw.length;
      const removed = raw.slice(0, end).length - keepPasswordText(raw.slice(0, end)).length;
      input.value = next;
      const caret = end - removed;
      input.setSelectionRange(caret, caret);
    }
    onChange?.(event);
  }

  return (
    <div className={clsx(styles.root, className)}>
      <input
        {...props}
        className={clsx(styles.input, inputClassName)}
        type={visible ? 'text' : 'password'}
        disabled={disabled}
        onBeforeInput={beforeInput}
        onChange={change}
      />
      <button
        type="button"
        className={styles.toggle}
        disabled={disabled}
        aria-label={visible ? '隐藏密码' : '显示密码'}
        aria-pressed={visible}
        onClick={() => setVisible((value) => !value)}
      >
        {visible ? (
          <EyeSlashIcon className={styles.icon} weight="regular" />
        ) : (
          <EyeIcon className={styles.icon} weight="regular" />
        )}
      </button>
    </div>
  );
}
