import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { dbCreate } from '@/bridge/db';
import { Button } from '@/components/button';
import { PasswordInput } from '@/components/password-input';
import { errorMessage, rawErrorMessage } from '@/utils/error';
import styles from './index.module.css';

/** 创建档案 Props */
type CreateViewProps = {
  /** 创建完成回调 */
  onDone: () => Promise<unknown>;
};

type ValidationCode = 'passwordShort' | 'passwordMismatch';

/** 创建表单上最近的一条错误 */
type CreateError = { field: 'password' | 'confirm'; code: ValidationCode } | { field: 'form'; message: string };

/** 校验密码和确认密码，返回稳定错误码 */
function validate(password: string, passwordConfirm: string): CreateError | null {
  if (password.length < 8) {
    return { field: 'password', code: 'passwordShort' };
  }
  if (password !== passwordConfirm) {
    return { field: 'confirm', code: 'passwordMismatch' };
  }
  return null;
}

/** 创建档案 */
export function CreateView({ onDone }: CreateViewProps) {
  const { t } = useTranslation('lock');
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
    return !busy && Boolean(password) && Boolean(passwordConfirm) && !error;
  }, [password, passwordConfirm, busy, error]);

  /** 把创建失败换成当前这一条错误 */
  function applyCreateError(message: string, display = message) {
    if (message === '密码至少 8 位' || message === '密码不能为空') {
      setError({ field: 'password', code: 'passwordShort' });
      return;
    }
    if (message === '两次输入的密码不一致') {
      setError({ field: 'confirm', code: 'passwordMismatch' });
      return;
    }
    setError({ field: 'form', message: display });
  }

  /** 离开密码框时检查长度 */
  function blurPassword() {
    if (busy) {
      return;
    }
    if (password.length < 8) {
      setError({ field: 'password', code: 'passwordShort' });
      return;
    }
    setError((current) => (current?.field === 'password' ? null : current));
  }

  /** 离开确认框时检查两次是否一致 */
  function blurConfirm() {
    if (busy) {
      return;
    }
    if (password !== passwordConfirm) {
      setError({ field: 'confirm', code: 'passwordMismatch' });
      return;
    }
    setError((current) => (current?.field === 'confirm' ? null : current));
  }

  /** 提交创建。两次都不通过时只留下密码那一条。 */
  async function submit() {
    const nextError = validate(password, passwordConfirm);
    if (nextError) {
      setError(nextError);
      return;
    }
    setError(null);

    setBusy(true);
    try {
      await dbCreate(password, passwordConfirm);
      await onDone();
    } catch (err) {
      applyCreateError(rawErrorMessage(err), errorMessage(err));
    } finally {
      setPassword('');
      setPasswordConfirm('');
      setBusy(false);
    }
  }

  const errorText = error == null ? '' : error.field === 'form' ? error.message : t(`errors.${error.code}`);

  return (
    <main className={styles.gate}>
      <h1>{t('create.title')}</h1>
      <p className={styles.muted}>{t('create.subtitle')}</p>
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
          <PasswordInput
            id="create-password"
            label={t('create.password')}
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
          <PasswordInput
            id="create-password-confirm"
            label={t('create.confirm')}
            autoComplete="new-password"
            value={passwordConfirm}
            disabled={busy}
            aria-invalid={error?.field === 'confirm' ? true : undefined}
            aria-describedby={error?.field === 'confirm' ? 'create-error' : undefined}
            onBlur={blurConfirm}
            onChange={(event) => {
              setPasswordConfirm(event.currentTarget.value);
              setError(null);
            }}
          />
        </div>
        <p id="create-error" className={styles.fieldError} aria-live="polite">
          {errorText}
        </p>
        <Button type="submit" className={styles.submit} disabled={!canSubmit}>
          {busy ? t('create.creating') : t('create.create')}
        </Button>
      </form>
    </main>
  );
}
