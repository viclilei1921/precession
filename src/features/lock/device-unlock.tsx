import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { dbDisableDeviceUnlock, dbEnableDeviceUnlock, dbStatus } from '@/bridge/db';
import { Button } from '@/components/button';
import { PasswordInput } from '@/components/password-input';
import { SettingRow } from '@/features/settings/row';
import { Switch } from '@/features/settings/switch';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import styles from './device-unlock.module.css';

export function DeviceUnlock() {
  const { t } = useTranslation('lock');
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
      setError(t('device.errorPasswordRequired'));
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

  const hint = deviceUnlock ? t('device.hintOn') : t('device.hintOff');

  return (
    <>
      <SettingRow title={t('device.rowTitle')} hint={hint}>
        <Switch label={t('device.label')} checked={deviceUnlock} disabled={busy} onClick={onSwitch} />
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
            label={t('device.password')}
            autoComplete="current-password"
            placeholder={t('device.placeholder')}
            value={password}
            disabled={busy}
            onChange={(event) => setPassword(event.currentTarget.value)}
          />
          {error ? <p className={styles.error}>{error}</p> : null}
          <div className={styles.actions}>
            <Button variant="text" disabled={busy} onClick={closeForm}>
              {t('device.cancel')}
            </Button>
            <Button type="submit" disabled={busy}>
              {busy ? t('device.enabling') : t('device.enable')}
            </Button>
          </div>
        </form>
      ) : null}
      {!asking && error ? <p className={styles.error}>{error}</p> : null}
    </>
  );
}
