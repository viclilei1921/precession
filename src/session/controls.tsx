import { useState } from 'react';
import { dbDisableDeviceUnlock, dbEnableDeviceUnlock } from '@/bridge/db';
import styles from './controls.module.css';
import { errorMessage } from './error';
import { useSession } from './store';

export function SessionControls() {
  const deviceUnlock = useSession((state) => state.status?.deviceUnlock ?? false);
  const refresh = useSession((state) => state.refresh);
  const lock = useSession((state) => state.lock);
  const [devicePassword, setDevicePassword] = useState('');
  const [deviceBusy, setDeviceBusy] = useState(false);
  const [locking, setLocking] = useState(false);
  const [error, setError] = useState('');

  async function enableDevice() {
    setDeviceBusy(true);
    setError('');
    try {
      await dbEnableDeviceUnlock(devicePassword);
      setDevicePassword('');
      await refresh();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setDeviceBusy(false);
    }
  }

  async function disableDevice() {
    setDeviceBusy(true);
    setError('');
    try {
      await dbDisableDeviceUnlock();
      await refresh();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setDeviceBusy(false);
    }
  }

  async function lockArchive() {
    setLocking(true);
    setError('');
    try {
      await lock();
    } catch (err) {
      setError(errorMessage(err));
      setLocking(false);
    }
  }

  return (
    <div className={styles.foot}>
      <p className={styles.hint}>
        {deviceUnlock
          ? '这台设备已启用免密打开。点锁定后需要系统验证。'
          : '可以启用设备解锁，以后打开不必输入档案密码。'}
      </p>
      {deviceUnlock ? (
        <button type="button" disabled={deviceBusy || locking} onClick={() => void disableDevice()}>
          {deviceBusy ? '正在关闭…' : '关闭设备解锁'}
        </button>
      ) : (
        <form
          onSubmit={(event) => {
            event.preventDefault();
            if (!deviceBusy && !locking) {
              void enableDevice();
            }
          }}
        >
          <label htmlFor="device-password">档案密码</label>
          <input
            id="device-password"
            type="password"
            autoComplete="current-password"
            value={devicePassword}
            disabled={deviceBusy || locking}
            onChange={(event) => setDevicePassword(event.currentTarget.value)}
          />
          <button type="submit" disabled={deviceBusy || locking}>
            {deviceBusy ? '正在启用…' : '启用设备解锁'}
          </button>
        </form>
      )}
      {error ? <p className={styles.error}>{error}</p> : null}
      <button type="button" disabled={deviceBusy || locking} onClick={() => void lockArchive()}>
        {locking ? '正在锁定…' : '锁定'}
      </button>
    </div>
  );
}
