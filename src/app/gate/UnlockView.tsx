import { useCallback, useEffect, useRef, useState } from 'react';
import { errorMessage } from '@/app/error';
import { dbDisableDeviceUnlock, dbUnlock, dbUnlockDevice } from '@/bridge/db';
import styles from './gate.module.css';

type UnlockViewProps = {
  deviceUnlock: boolean;
  held: boolean;
  onDone: () => Promise<unknown>;
};

export function UnlockView({ deviceUnlock, held, onDone }: UnlockViewProps) {
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [showPassword, setShowPassword] = useState(!deviceUnlock);
  const attempted = useRef(false);
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
    if (held || !deviceUnlock || attempted.current) {
      return;
    }
    attempted.current = true;
    void unlockWithDevice();
  }, [deviceUnlock, held, unlockWithDevice]);

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
            ? held
              ? '可以输入档案密码，也可以再试一次系统验证。'
              : '可以输入档案密码，也可以直接解锁。'
            : '输入密码以打开人生档案。'
          : held
            ? '已锁定。点解锁后需要通过系统验证。'
            : '正在打开档案…'}
      </p>
      {busy && !showPassword ? <p className={styles.muted}>正在解锁…</p> : null}
      {!showPassword && held ? (
        <form
          className={styles.form}
          onSubmit={(event) => {
            event.preventDefault();
            if (!busy) {
              void unlockWithDevice();
            }
          }}
        >
          {error ? <p className={styles.error}>{error}</p> : null}
          <button type="submit" disabled={busy}>
            {busy ? '正在解锁…' : '解锁'}
          </button>
        </form>
      ) : null}
      {showPassword ? (
        <form
          className={styles.form}
          onSubmit={(event) => {
            event.preventDefault();
            if (!busy) {
              void submit();
            }
          }}
        >
          <label htmlFor="unlock-password">密码</label>
          <input
            id="unlock-password"
            type="password"
            autoComplete="current-password"
            value={password}
            disabled={busy}
            onChange={(event) => setPassword(event.currentTarget.value)}
          />
          {error ? <p className={styles.error}>{error}</p> : null}
          <button type="submit" disabled={busy}>
            {busy ? '正在解锁…' : '用档案密码解锁'}
          </button>
          {deviceUnlock ? (
            <button
              type="button"
              disabled={busy}
              onClick={() => {
                void unlockWithDevice();
              }}
            >
              {held ? '用系统验证解锁' : '直接解锁'}
            </button>
          ) : null}
          {error === '设备解锁已失效' ? (
            <button
              type="button"
              disabled={busy}
              onClick={() => {
                void removeDevice();
              }}
            >
              移除设备解锁
            </button>
          ) : null}
        </form>
      ) : null}
    </main>
  );
}
