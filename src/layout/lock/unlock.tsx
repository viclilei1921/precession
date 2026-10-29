import { useCallback, useEffect, useRef, useState } from 'react';
import { dbDisableDeviceUnlock, dbUnlock, dbUnlockDevice } from '@/bridge/db';
import { Button } from '@/components/button';
import { PasswordInput } from '@/components/password-input';
import { errorMessage } from '@/utils/error';
import styles from './index.module.css';

/** 密码框上的错误。其余错误属于整张表单。 */
function isPasswordError(message: string) {
  return message === '密码不正确' || message === '密码不能为空' || message === '密码至少 8 位';
}

/** 解锁档案 Props */
type UnlockViewProps = {
  /** 是否设备解锁 */
  deviceUnlock: boolean;
  /** 是否用户锁定 */
  userLocked: boolean;
  /** 完成回调 */
  onDone: () => Promise<unknown>;
};

/** 解锁档案 */
export function UnlockView({ deviceUnlock, userLocked, onDone }: UnlockViewProps) {
  /** 密码 */
  const [password, setPassword] = useState('');
  /** 错误 */
  const [error, setError] = useState('');
  /** 是否正在解锁 */
  const [busy, setBusy] = useState(false);
  /** 是否显示密码 */
  const [showPassword, setShowPassword] = useState(!deviceUnlock);
  /** 是否尝试过解锁 */
  const attempted = useRef(false);
  /** 完成回调引用 */
  const onDoneRef = useRef(onDone);
  onDoneRef.current = onDone;

  const unlockWithDevice = useCallback(async () => {
    setBusy(true);
    setError('');
    try {
      await dbUnlockDevice();
      await onDoneRef.current();
    } catch (err) {
      setError(errorMessage(err));
      setShowPassword(true);
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => {
    if (userLocked || !deviceUnlock || attempted.current) {
      return;
    }
    attempted.current = true;
    void unlockWithDevice();
  }, [deviceUnlock, userLocked, unlockWithDevice]);

  async function submit() {
    setBusy(true);
    setError('');
    try {
      await dbUnlock(password);
      await onDone();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setPassword('');
      setBusy(false);
    }
  }

  async function removeDevice() {
    setBusy(true);
    setError('');
    try {
      await dbDisableDeviceUnlock();
      setShowPassword(true);
      await onDone();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className={styles.gate}>
      <h1>解锁档案</h1>
      <p className={styles.muted}>
        {showPassword
          ? deviceUnlock
            ? userLocked
              ? '可以输入档案密码，也可以再试一次系统验证。'
              : '可以输入档案密码，也可以直接解锁。'
            : '输入密码以打开人生档案。'
          : userLocked
            ? '已锁定。点解锁后需要通过系统验证。'
            : '正在打开档案…'}
      </p>
      {busy && !showPassword ? <p className={styles.muted}>正在解锁…</p> : null}
      {!showPassword && userLocked ? (
        <form
          className={styles.form}
          aria-describedby={error ? 'unlock-error' : undefined}
          onSubmit={(event) => {
            event.preventDefault();
            if (!busy) {
              void unlockWithDevice();
            }
          }}
        >
          <p id="unlock-error" className={styles.fieldError} aria-live="polite">
            {error}
          </p>
          <Button type="submit" className={styles.submit} disabled={busy}>
            {busy ? '正在解锁…' : '解锁'}
          </Button>
        </form>
      ) : null}
      {showPassword ? (
        <form
          className={styles.form}
          aria-describedby={error && !isPasswordError(error) ? 'unlock-error' : undefined}
          onSubmit={(event) => {
            event.preventDefault();
            if (!busy) {
              void submit();
            }
          }}
        >
          <div className={styles.field}>
            <PasswordInput
              id="unlock-password"
              label="密码"
              autoComplete="current-password"
              value={password}
              disabled={busy}
              aria-invalid={isPasswordError(error) ? true : undefined}
              aria-describedby={isPasswordError(error) ? 'unlock-error' : undefined}
              onChange={(event) => {
                setPassword(event.currentTarget.value);
                setError((current) => (isPasswordError(current) ? '' : current));
              }}
            />
          </div>
          <p id="unlock-error" className={styles.fieldError} aria-live="polite">
            {error}
          </p>
          <Button type="submit" className={styles.submit} disabled={busy}>
            {busy ? '正在解锁…' : '用档案密码解锁'}
          </Button>
          {deviceUnlock ? (
            <Button
              variant="outlined"
              className={styles.submit}
              disabled={busy}
              onClick={() => {
                void unlockWithDevice();
              }}
            >
              {userLocked ? '用系统验证解锁' : '直接解锁'}
            </Button>
          ) : null}
          {error === '设备解锁已失效' ? (
            <Button
              variant="outlined"
              tone="destructive"
              className={styles.submit}
              disabled={busy}
              onClick={() => {
                void removeDevice();
              }}
            >
              移除设备解锁
            </Button>
          ) : null}
        </form>
      ) : null}
    </main>
  );
}
