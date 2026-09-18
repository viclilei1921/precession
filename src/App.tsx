import { useEffect, useState } from 'react';
import type { DbStatus } from './bridge/db';
import { dbCreate, dbLock, dbStatus, dbUnlock } from './bridge/db';
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
    return <UnlockView onDone={refreshStatus} />;
  }

  return <UnlockedView onLocked={refreshStatus} />;
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

function UnlockView({ onDone }: { onDone: () => Promise<unknown> }) {
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);

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

  return (
    <main className="gate">
      <h1>解锁档案</h1>
      <p className="gate-muted">输入密码以打开人生档案。</p>
      <form
        className="gate-form"
        onSubmit={(e) => {
          e.preventDefault();
          if (!busy) {
            submit();
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
          {busy ? '正在解锁…' : '解锁'}
        </button>
      </form>
    </main>
  );
}

function UnlockedView({ onLocked }: { onLocked: () => Promise<unknown> }) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');

  async function lock() {
    setBusy(true);
    setError('');
    try {
      await dbLock();
      await onLocked();
    } catch (err) {
      setError(errorMessage(err));
      setBusy(false);
    }
  }

  return (
    <main className="gate">
      <h1>库已解锁</h1>
      <p className="gate-muted">关窗进托盘不会要求重新输入密码。锁定或退出后需要再输入。</p>
      {error ? <p className="gate-error">{error}</p> : null}
      <button type="button" disabled={busy} onClick={lock}>
        {busy ? '正在锁定…' : '锁定'}
      </button>
    </main>
  );
}

export default App;
