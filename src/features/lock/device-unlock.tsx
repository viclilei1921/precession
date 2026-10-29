import { useState } from 'react';
import { dbDisableDeviceUnlock, dbEnableDeviceUnlock, dbStatus } from '@/bridge/db';
import { Button } from '@/components/button';
import { PasswordInput } from '@/components/password-input';
import { SettingRow } from '@/features/settings/row';
import { Switch } from '@/features/settings/switch';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import styles from './device-unlock.module.css';

export function DeviceUnlock() {
  const deviceUnlock = useAppStore((state) => state.dbStatus?.deviceUnlock ?? false);
  const setDbStatus = useAppStore((state) => state.setDbStatus);
  const [password, setPassword] = useState('');
  const [asking, setAsking] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');

  async function refresh() {
    setDbStatus(await dbStatus());
  }

  function closeForm() {
    setAsking(false);
    setPassword('');
    setError('');
  }

  async function enableDevice() {
    if (!password) {
      setError('请输入档案密码。');
      return;
    }
    setBusy(true);
    setError('');
    try {
      await dbEnableDeviceUnlock(password);
      closeForm();
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
      closeForm();
      await refresh();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  function onSwitch() {
    if (busy) {
      return;
    }
    if (deviceUnlock) {
      void disableDevice();
      return;
    }
    if (asking) {
      closeForm();
      return;
    }
    setAsking(true);
  }

  const hint = deviceUnlock ? '已启用 · 打开应用用系统验证代替档案密码。' : '启用后，打开应用用系统验证代替档案密码。';

  return (
    <>
      <SettingRow title="设备解锁" hint={hint}>
        <Switch label="设备解锁" checked={deviceUnlock} disabled={busy} onClick={onSwitch} />
      </SettingRow>
      {asking && !deviceUnlock ? (
        <form
          className={styles.form}
          onSubmit={(event) => {
            event.preventDefault();
            if (!busy) {
              void enableDevice();
            }
          }}
        >
          <PasswordInput
            id="device-password"
            label="档案密码"
            autoComplete="current-password"
            placeholder="输入档案密码"
            value={password}
            disabled={busy}
            onChange={(event) => setPassword(event.currentTarget.value)}
          />
          {error ? <p className={styles.error}>{error}</p> : null}
          <div className={styles.actions}>
            <Button variant="text" disabled={busy} onClick={closeForm}>
              取消
            </Button>
            <Button type="submit" disabled={busy}>
              {busy ? '正在启用…' : '启用'}
            </Button>
          </div>
        </form>
      ) : null}
      {!asking && error ? <p className={styles.error}>{error}</p> : null}
    </>
  );
}
