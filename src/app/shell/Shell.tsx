import { Link, Outlet } from '@tanstack/react-router';
import { useState } from 'react';
import { errorMessage } from '@/app/error';
import { useSession } from '@/app/session';
import { dbDisableDeviceUnlock, dbEnableDeviceUnlock } from '@/bridge/db';
import styles from './shell.module.css';

const recordLinks = [
  ['/today', '今天'],
  ['/plan', '计划'],
  ['/journal', '手记'],
  ['/growth', '成长'],
  ['/library', '书库']
] as const;

const organizeLinks = [
  ['/toolbox', '工具箱'],
  ['/review', '回顾']
] as const;

export function Shell() {
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
    <div className={styles.shell}>
      <aside className={styles.nav}>
        <p className={styles.brand}>Precession</p>
        <nav className={styles.group} aria-label="记录">
          <div className={styles.label}>记录</div>
          {recordLinks.map(([to, label]) => (
            <Link key={to} to={to} className={styles.link}>
              {label}
            </Link>
          ))}
        </nav>
        <nav className={styles.group} aria-label="整理">
          <div className={styles.label}>整理</div>
          {organizeLinks.map(([to, label]) => (
            <Link key={to} to={to} className={styles.link}>
              {label}
            </Link>
          ))}
        </nav>
        <nav className={styles.group} aria-label="系统">
          <div className={styles.label}>系统</div>
          <Link to="/settings" className={styles.link}>
            设置
          </Link>
        </nav>
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
      </aside>
      <div className={styles.main}>
        <header className={styles.topbar}>
          <input className={styles.search} readOnly placeholder="搜索全部记录" aria-label="搜索全部记录" />
          <button type="button" className={styles.capture}>
            记一笔
          </button>
        </header>
        <div className={styles.content}>
          <Outlet />
        </div>
      </div>
    </div>
  );
}
