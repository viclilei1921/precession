import { useCallback, useEffect, useRef, useState } from 'react';
import type { DbStatus } from './bridge/db';
import {
  dbCreate,
  dbDisableDeviceUnlock,
  dbEnableDeviceUnlock,
  dbLock,
  dbStatus,
  dbUnlock,
  dbUnlockDevice
} from './bridge/db';
import type { DemoItem } from './bridge/demo';
import { demoCreate, demoDelete, demoList, demoUpdate } from './bridge/demo';
import './App.css';

function errorMessage(error: unknown): string {
  if (typeof error === 'string' && error.length > 0) {
    return error;
  }
  if (error instanceof Error && error.message) {
    return error.message;
  }
  if (error && typeof error === 'object' && 'message' in error && typeof error.message === 'string' && error.message) {
    return error.message;
  }
  return '操作失败';
}

function App() {
  const [status, setStatus] = useState<DbStatus | null>(null);
  const [bootError, setBootError] = useState('');

  async function refreshStatus() {
    const next = await dbStatus();
    setStatus(next);
    return next;
  }

  useEffect(() => {
    let cancelled = false;
    dbStatus()
      .then((next) => {
        if (!cancelled) {
          setStatus(next);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setBootError(errorMessage(error));
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  if (bootError) {
    return (
      <main className="gate">
        <h1>人生档案</h1>
        <p className="gate-error">{bootError}</p>
      </main>
    );
  }

  if (!status) {
    return (
      <main className="gate">
        <h1>人生档案</h1>
        <p className="gate-muted">加载中…</p>
      </main>
    );
  }

  if (!status.exists) {
    return <CreateView onDone={refreshStatus} />;
  }

  if (!status.unlocked) {
    return <UnlockView deviceUnlock={status.deviceUnlock} held={status.userLocked} onDone={refreshStatus} />;
  }

  return <UnlockedView deviceUnlock={status.deviceUnlock} onChanged={refreshStatus} />;
}

function CreateView({ onDone }: { onDone: () => Promise<unknown> }) {
  const [password, setPassword] = useState('');
  const [passwordConfirm, setPasswordConfirm] = useState('');
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);

  async function submit() {
    setBusy(true);
    setError('');
    try {
      await dbCreate(password, passwordConfirm);
      await onDone();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setPassword('');
      setPasswordConfirm('');
      setBusy(false);
    }
  }

  return (
    <main className="gate">
      <h1>设置档案密码</h1>
      <p className="gate-muted">第一次打开需要设置密码。密码至少 8 位。</p>
      <form
        className="gate-form"
        onSubmit={(e) => {
          e.preventDefault();
          if (!busy) {
            submit();
          }
        }}
      >
        <label htmlFor="create-password">密码</label>
        <input
          id="create-password"
          type="password"
          autoComplete="new-password"
          value={password}
          disabled={busy}
          onChange={(e) => setPassword(e.currentTarget.value)}
        />
        <label htmlFor="create-password-confirm">再输入一次</label>
        <input
          id="create-password-confirm"
          type="password"
          autoComplete="new-password"
          value={passwordConfirm}
          disabled={busy}
          onChange={(e) => setPasswordConfirm(e.currentTarget.value)}
        />
        {error ? <p className="gate-error">{error}</p> : null}
        <button type="submit" disabled={busy}>
          {busy ? '正在创建…' : '创建档案'}
        </button>
      </form>
    </main>
  );
}

function UnlockView({
  deviceUnlock,
  held,
  onDone
}: {
  deviceUnlock: boolean;
  held: boolean;
  onDone: () => Promise<unknown>;
}) {
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
    <main className="gate">
      <h1>解锁档案</h1>
      <p className="gate-muted">
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
      {busy && !showPassword ? <p className="gate-muted">正在解锁…</p> : null}
      {!showPassword && held ? (
        <form
          className="gate-form"
          onSubmit={(e) => {
            e.preventDefault();
            if (!busy) {
              void unlockWithDevice();
            }
          }}
        >
          {error ? <p className="gate-error">{error}</p> : null}
          <button type="submit" disabled={busy}>
            {busy ? '正在解锁…' : '解锁'}
          </button>
        </form>
      ) : null}
      {showPassword ? (
        <form
          className="gate-form"
          onSubmit={(e) => {
            e.preventDefault();
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
            onChange={(e) => setPassword(e.currentTarget.value)}
          />
          {error ? <p className="gate-error">{error}</p> : null}
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

function UnlockedView({ deviceUnlock, onChanged }: { deviceUnlock: boolean; onChanged: () => Promise<unknown> }) {
  const [items, setItems] = useState<DemoItem[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [title, setTitle] = useState('');
  const [body, setBody] = useState('');
  const [busy, setBusy] = useState(false);
  const [locking, setLocking] = useState(false);
  const [error, setError] = useState('');
  const [loadError, setLoadError] = useState('');
  const [devicePassword, setDevicePassword] = useState('');
  const [deviceBusy, setDeviceBusy] = useState(false);
  const [deviceError, setDeviceError] = useState('');

  async function refreshList() {
    const next = await demoList();
    setItems(next);
    return next;
  }

  useEffect(() => {
    let cancelled = false;
    demoList()
      .then((next) => {
        if (!cancelled) {
          setItems(next);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setLoadError(errorMessage(err));
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  function resetForm() {
    setSelectedId(null);
    setTitle('');
    setBody('');
  }

  function selectItem(item: DemoItem) {
    setSelectedId(item.id);
    setTitle(item.title);
    setBody(item.body);
    setError('');
  }

  async function save() {
    setBusy(true);
    setError('');
    try {
      if (selectedId) {
        await demoUpdate(selectedId, title, body);
      } else {
        await demoCreate(title, body);
      }
      await refreshList();
      resetForm();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  async function remove(id: string) {
    setBusy(true);
    setError('');
    try {
      await demoDelete(id);
      if (selectedId === id) {
        resetForm();
      }
      await refreshList();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  async function enableDevice() {
    setDeviceBusy(true);
    setDeviceError('');
    try {
      await dbEnableDeviceUnlock(devicePassword);
      setDevicePassword('');
      await onChanged();
    } catch (err) {
      setDeviceError(errorMessage(err));
    } finally {
      setDeviceBusy(false);
    }
  }

  async function disableDevice() {
    setDeviceBusy(true);
    setDeviceError('');
    try {
      await dbDisableDeviceUnlock();
      await onChanged();
    } catch (err) {
      setDeviceError(errorMessage(err));
    } finally {
      setDeviceBusy(false);
    }
  }

  async function lock() {
    setLocking(true);
    setError('');
    try {
      await dbLock();
      await onChanged();
    } catch (err) {
      setError(errorMessage(err));
      setLocking(false);
    }
  }

  return (
    <main className="playground">
      <header className="playground-header">
        <div>
          <h1>示例表 CRUD</h1>
          <p className="gate-muted">
            {deviceUnlock
              ? '再次打开应用会直接进入。点锁定后会保持锁定，重启也一样，解锁需要系统验证。'
              : '锁定或退出后需要再输入密码。可以启用设备解锁，以后打开应用不必输入档案密码。'}
          </p>
        </div>
        <button type="button" disabled={busy || locking} onClick={lock}>
          {locking ? '正在锁定…' : '锁定'}
        </button>
      </header>

      <form
        className="gate-form playground-form"
        onSubmit={(e) => {
          e.preventDefault();
          if (!deviceBusy && !busy && !locking) {
            if (deviceUnlock) {
              void disableDevice();
            } else {
              void enableDevice();
            }
          }
        }}
      >
        {deviceUnlock ? (
          <button type="submit" disabled={deviceBusy || busy || locking}>
            {deviceBusy ? '正在关闭…' : '关闭设备解锁'}
          </button>
        ) : (
          <>
            <label htmlFor="device-password">档案密码</label>
            <input
              id="device-password"
              type="password"
              autoComplete="current-password"
              value={devicePassword}
              disabled={deviceBusy || busy || locking}
              onChange={(e) => setDevicePassword(e.currentTarget.value)}
            />
            <button type="submit" disabled={deviceBusy || busy || locking}>
              {deviceBusy ? '正在启用…' : '启用设备解锁'}
            </button>
          </>
        )}
        {deviceError ? <p className="gate-error">{deviceError}</p> : null}
      </form>

      {loadError ? <p className="gate-error">{loadError}</p> : null}

      <form
        className="gate-form playground-form"
        onSubmit={(e) => {
          e.preventDefault();
          if (!busy && !locking) {
            save();
          }
        }}
      >
        <label htmlFor="demo-title">标题</label>
        <input
          id="demo-title"
          value={title}
          disabled={busy || locking}
          onChange={(e) => setTitle(e.currentTarget.value)}
        />
        <label htmlFor="demo-body">正文</label>
        <textarea
          id="demo-body"
          rows={4}
          value={body}
          disabled={busy || locking}
          onChange={(e) => setBody(e.currentTarget.value)}
        />
        {error ? <p className="gate-error">{error}</p> : null}
        <div className="playground-actions">
          <button type="submit" disabled={busy || locking}>
            {selectedId ? '保存修改' : '新建'}
          </button>
          {selectedId ? (
            <button type="button" disabled={busy || locking} onClick={resetForm}>
              取消编辑
            </button>
          ) : null}
        </div>
      </form>

      {items.length === 0 ? (
        <p className="gate-muted">还没有记录。</p>
      ) : (
        <ul className="demo-list">
          {items.map((item) => (
            <li key={item.id} className={item.id === selectedId ? 'demo-item demo-item-active' : 'demo-item'}>
              <button
                type="button"
                className="demo-item-pick"
                disabled={busy || locking}
                onClick={() => selectItem(item)}
              >
                <strong>{item.title}</strong>
                {item.body ? <span>{item.body}</span> : null}
              </button>
              <button
                type="button"
                className="demo-item-delete"
                disabled={busy || locking}
                onClick={() => remove(item.id)}
              >
                删除
              </button>
            </li>
          ))}
        </ul>
      )}
    </main>
  );
}

export default App;
