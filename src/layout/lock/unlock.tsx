import { useCallback, useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { dbDisableDeviceUnlock, dbUnlock, dbUnlockDevice } from '@/bridge/db';
import { Button } from '@/components/button';
import { PasswordInput } from '@/components/password-input';
import { errorMessage } from '@/utils/error';
import styles from './index.module.css';

/** 取出后端原始错误文本，用来判断错误类型（展示仍走 errorMessage）。 */
function rawMessage(error: unknown): string {
  if (typeof error === 'string') {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  if (error && typeof error === 'object' && 'message' in error && typeof error.message === 'string') {
    return error.message;
  }
  return '';
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
  const { t } = useTranslation('lock');
  /** 密码 */
  const [password, setPassword] = useState('');
  /** 错误 */
  const [error, setError] = useState('');
  /** 是否正在解锁 */
  const [busy, setBusy] = useState(false);
  /** 是否显示密码 */
  const [showPassword, setShowPassword] = useState(!deviceUnlock);
  /** 密码框是否要标红。由提交结果决定，不再比对本地化后的文案 */
  const [passwordInvalid, setPasswordInvalid] = useState(false);
  /** 设备解锁是否已失效 */
  const [deviceExpired, setDeviceExpired] = useState(false);
  /** 是否尝试过解锁 */
  const attempted = useRef(false);
  /** 完成回调引用 */
  const onDoneRef = useRef(onDone);
  onDoneRef.current = onDone;

  const unlockWithDevice = useCallback(async () => {
    setBusy(true);
    setError('');
    setPasswordInvalid(false);
    try {
      await dbUnlockDevice();
      await onDoneRef.current();
    } catch (err) {
      setDeviceExpired(rawMessage(err) === '设备解锁已失效');
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
    setPasswordInvalid(false);
    try {
      await dbUnlock(password);
      await onDone();
    } catch (err) {
      setPasswordInvalid(true);
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

  const subtitle = showPassword
    ? deviceUnlock
      ? userLocked
        ? t('unlock.subtitleDeviceLocked')
        : t('unlock.subtitleDevice')
      : t('unlock.subtitlePassword')
    : userLocked
      ? t('unlock.subtitleLocked')
      : t('unlock.subtitleOpening');

  return (
    <main className={styles.gate}>
      <h1>{t('unlock.title')}</h1>
      <p className={styles.muted}>{subtitle}</p>
      {busy && !showPassword ? <p className={styles.muted}>{t('unlock.unlocking')}</p> : null}
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
            {busy ? t('unlock.unlocking') : t('unlock.unlock')}
          </Button>
        </form>
      ) : null}
      {showPassword ? (
        <form
          className={styles.form}
          aria-describedby={error && !passwordInvalid ? 'unlock-error' : undefined}
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
              label={t('unlock.password')}
              autoComplete="current-password"
              value={password}
              disabled={busy}
              aria-invalid={passwordInvalid ? true : undefined}
              aria-describedby={passwordInvalid ? 'unlock-error' : undefined}
              onChange={(event) => {
                setPassword(event.currentTarget.value);
                setPasswordInvalid(false);
                setError((current) => (passwordInvalid ? '' : current));
              }}
            />
          </div>
          <p id="unlock-error" className={styles.fieldError} aria-live="polite">
            {error}
          </p>
          <Button type="submit" className={styles.submit} disabled={busy}>
            {busy ? t('unlock.unlocking') : t('unlock.unlockWithPassword')}
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
              {userLocked ? t('unlock.unlockWithDevice') : t('unlock.unlockDirect')}
            </Button>
          ) : null}
          {deviceExpired ? (
            <Button
              variant="outlined"
              tone="destructive"
              className={styles.submit}
              disabled={busy}
              onClick={() => {
                void removeDevice();
              }}
            >
              {t('unlock.removeDevice')}
            </Button>
          ) : null}
        </form>
      ) : null}
    </main>
  );
}
