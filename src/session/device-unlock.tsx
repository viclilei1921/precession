import { useState } from 'react';
import { dbDisableDeviceUnlock, dbEnableDeviceUnlock } from '@/bridge/db';
import { errorMessage } from '@/shared/lib/error';
import { useSession } from '@/store/session';
import styles from './session.module.css';

export function DeviceUnlock() {
  const deviceUnlock = useSession((state) => state.status?.deviceUnlock ?? false);
  const refresh = useSession((state) => state.refresh);
  const [password, setPassword] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');

  async function enableDevice() {
    setBusy(true);
    setError('');
    try {
      await dbEnableDeviceUnlock(password);
      setPassword('');
      await refresh();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  async function disableDevice() {
    setBusy(true);
    setError('');
    try {
      await dbDisableDeviceUnlock();
      await refresh();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className={styles.device}>
      <h2>设备解锁</h2>
      <p className={styles.hint}>
        {deviceUnlock ? '这台设备已启用免密打开。点锁定后需要系统验证。' : '启用后，打开应用不必输入档案密码。'}
      </p>
      {deviceUnlock ? (
        <button type="button" disabled={busy} onClick={() => void disableDevice()}>
          {busy ? '正在关闭…' : '关闭设备解锁'}
        </button>
      ) : (
        <form
          className={styles.form}
          onSubmit={(event) => {
            event.preventDefault();
            if (!busy) {
              void enableDevice();
            }
          }}
        >
          <label htmlFor="device-password">档案密码</label>
          <input
            id="device-password"
            type="password"
            autoComplete="current-password"
            value={password}
            disabled={busy}
            onChange={(event) => setPassword(event.currentTarget.value)}
          />
          <button type="submit" disabled={busy}>
            {busy ? '正在启用…' : '启用设备解锁'}
          </button>
        </form>
      )}
      {error ? <p className={styles.error}>{error}</p> : null}
    </section>
  );
}
