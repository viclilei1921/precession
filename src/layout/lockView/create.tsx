import { useMemo, useState } from 'react';
import { dbCreate } from '@/bridge/db';
import { PasswordInput } from '@/components/passwordInput';
import { errorMessage } from '@/utils/error';
import styles from './index.module.css';

/** 创建档案 Props */
type CreateViewProps = {
  /** 创建完成回调 */
  onDone: () => Promise<unknown>;
};

/** 创建档案 */
export function CreateView({ onDone }: CreateViewProps) {
  /** 密码 */
  const [password, setPassword] = useState('');
  /** 确认密码 */
  const [passwordConfirm, setPasswordConfirm] = useState('');
  /** 密码错误 */
  const [passwordError, setPasswordError] = useState('');
  /** 确认密码错误 */
  const [confirmError, setConfirmError] = useState('');
  /** 与输入无关的错误 */
  const [formError, setFormError] = useState('');
  /** 是否正在创建 */
  const [busy, setBusy] = useState(false);

  /** 是否可以提交 */
  const canSubmit = useMemo(() => {
    return !busy && password && passwordConfirm;
  }, [password, passwordConfirm, busy]);

  /** 把创建失败对应到具体输入框 */
  function applyCreateError(message: string) {
    if (message === '密码至少 8 位' || message === '密码不能为空') {
      setPasswordError(message);
      return;
    }
    if (message === '两次输入的密码不一致') {
      setConfirmError(message);
      return;
    }
    setFormError(message);
  }

  /** 提交创建 */
  async function submit() {
    const nextPasswordError = password.length < 8 ? '密码至少 8 位' : '';
    const nextConfirmError = password !== passwordConfirm ? '两次输入的密码不一致' : '';
    setPasswordError(nextPasswordError);
    setConfirmError(nextConfirmError);
    setFormError('');
    if (nextPasswordError || nextConfirmError) {
      return;
    }

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
            aria-invalid={passwordError ? true : undefined}
            aria-describedby={passwordError ? 'create-password-error' : undefined}
            onChange={(event) => {
              setPassword(event.currentTarget.value);
              setPasswordError('');
              setConfirmError('');
              setFormError('');
            }}
          />
          {passwordError ? (
            <p id="create-password-error" className={styles.fieldError}>
              {passwordError}
            </p>
          ) : null}
        </div>
        <div className={styles.field}>
          <label htmlFor="create-password-confirm">再输入一次</label>
          <PasswordInput
            id="create-password-confirm"
            autoComplete="new-password"
            value={passwordConfirm}
            disabled={busy}
            aria-invalid={confirmError ? true : undefined}
            aria-describedby={confirmError ? 'create-password-confirm-error' : undefined}
            onChange={(event) => {
              setPasswordConfirm(event.currentTarget.value);
              setConfirmError('');
              setFormError('');
            }}
          />
          {confirmError ? (
            <p id="create-password-confirm-error" className={styles.fieldError}>
              {confirmError}
            </p>
          ) : null}
        </div>
        {formError ? <p className={styles.fieldError}>{formError}</p> : null}
        <button type="submit" disabled={!canSubmit}>
          {busy ? '正在创建…' : '创建档案'}
        </button>
      </form>
    </div>
  );
}
