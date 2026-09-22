import { useEffect, useState } from 'react';
import type { DbStatus } from './bridge/db';
import { dbCreate, dbLock, dbStatus, dbUnlock } from './bridge/db';
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
  const [items, setItems] = useState<DemoItem[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [title, setTitle] = useState('');
  const [body, setBody] = useState('');
  const [busy, setBusy] = useState(false);
  const [locking, setLocking] = useState(false);
  const [error, setError] = useState('');
  const [loadError, setLoadError] = useState('');

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

  async function lock() {
    setLocking(true);
    setError('');
    try {
      await dbLock();
      await onLocked();
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
          <p className="gate-muted">验证加密库可读写。锁定或退出后需要再输入密码。</p>
        </div>
        <button type="button" disabled={busy || locking} onClick={lock}>
          {locking ? '正在锁定…' : '锁定'}
        </button>
      </header>

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
