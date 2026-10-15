import { useMemo, useState } from 'react';
import { dbCreate } from '@/bridge/db';
import { PasswordInput } from '@/components/password-input';
import { errorMessage } from '@/utils/error';
import styles from './index.module.css';

/** 创建档案 Props */
type CreateViewProps = {
  /** 创建完成回调 */
  onDone: () => Promise<unknown>;
};

/** 创建表单上最近的一条错误 */
type CreateError = {
  /** 错误对应哪一项 */
  field: 'password' | 'confirm' | 'form';
  /** 错误文案 */
  message: string;
};

/** 密码至少 8 位。空密码同样不通过。 */
function passwordErrorMessage(password: string) {
  return password.length < 8 ? '密码至少 8 位' : '';
}

/** 两次输入不一致时的文案 */
function confirmErrorMessage(password: string, passwordConfirm: string) {
  return password !== passwordConfirm ? '两次输入的密码不一致' : '';
}

/** 创建档案 */
export function CreateView({ onDone }: CreateViewProps) {
  /** 密码 */
  const [password, setPassword] = useState('');
  /** 确认密码 */
  const [passwordConfirm, setPasswordConfirm] = useState('');
  /** 只保留最近一次错误 */
  const [error, setError] = useState<CreateError | null>(null);
  /** 是否正在创建 */
  const [busy, setBusy] = useState(false);

  /** 是否可以提交 */
  const canSubmit = useMemo(() => {
    return !busy && password && passwordConfirm && !error;
  }, [password, passwordConfirm, busy, error]);

  /** 把创建失败换成当前这一条错误 */
  function applyCreateError(message: string) {
    if (message === '密码至少 8 位' || message === '密码不能为空') {
      setError({ field: 'password', message });
      return;
    }
    if (message === '两次输入的密码不一致') {
      setError({ field: 'confirm', message });
      return;
    }
    setError({ field: 'form', message });
  }

  /** 离开密码框时检查长度 */
  function blurPassword() {
    if (busy) {
      return;
    }
    const message = passwordErrorMessage(password);
    if (message) {
      setError({ field: 'password', message });
      return;
    }
    setError((current) => (current?.field === 'password' ? null : current));
  }

  /** 离开确认框时检查两次是否一致 */
  function blurConfirm() {
    if (busy) {
      return;
    }
    const message = confirmErrorMessage(password, passwordConfirm);
    if (message) {
      setError({ field: 'confirm', message });
      return;
    }
    setError((current) => (current?.field === 'confirm' ? null : current));
  }

  /** 提交创建。两次都不通过时只留下密码那一条。 */
  async function submit() {
    const nextPasswordError = passwordErrorMessage(password);
    if (nextPasswordError) {
      setError({ field: 'password', message: nextPasswordError });
      return;
    }
    const nextConfirmError = confirmErrorMessage(password, passwordConfirm);
    if (nextConfirmError) {
      setError({ field: 'confirm', message: nextConfirmError });
      return;
    }
    setError(null);

    setBusy(true);
    try {
      await dbCreate(password, passwordConfirm);
      await onDone();
    } catch (err) {
      applyCreateError(errorMessage(err));
    } finally {
      setPassword('');
      setPasswordConfirm('');
      setBusy(false);
    }
  }

  return (
    <div className={styles.gate}>
      <h1>设置档案密码</h1>
      <p className={styles.muted}>第一次打开需要设置密码。密码至少 8 位。</p>
      <form
        className={styles.form}
        aria-describedby={error?.field === 'form' ? 'create-error' : undefined}
        onSubmit={(event) => {
          event.preventDefault();
          if (!busy) {
            void submit();
          }
        }}
      >
        <div className={styles.field}>
          <label htmlFor="create-password">密码</label>
          <PasswordInput
            id="create-password"
            autoComplete="new-password"
            value={password}
            disabled={busy}
            aria-invalid={error?.field === 'password' ? true : undefined}
            aria-describedby={error?.field === 'password' ? 'create-error' : undefined}
            onBlur={blurPassword}
            onChange={(event) => {
              setPassword(event.currentTarget.value);
              setError(null);
            }}
          />
        </div>
        <div className={styles.field}>
          <label htmlFor="create-password-confirm">再输入一次</label>
          <PasswordInput
            id="create-password-confirm"
            autoComplete="new-password"
            value={passwordConfirm}
            disabled={busy}
            aria-invalid={error?.field === 'confirm' ? true : undefined}
            aria-describedby={error?.field === 'confirm' ? 'create-error' : undefined}
            onBlur={blurConfirm}
            onChange={(event) => {
              setPasswordConfirm(event.currentTarget.value);
              setError((current) => (current?.field === 'password' ? current : null));
            }}
          />
        </div>
        <p id="create-error" className={styles.fieldError} aria-live="polite">
          {error?.message ?? ''}
        </p>
        <button type="submit" disabled={!canSubmit}>
          {busy ? '正在创建…' : '创建档案'}
        </button>
      </form>
    </div>
  );
}
