import { useState } from 'react';
import { errorMessage } from '@/app/error';
import { dbCreate } from '@/bridge/db';
import styles from './gate.module.css';

type CreateViewProps = {
  onDone: () => Promise<unknown>;
};

export function CreateView({ onDone }: CreateViewProps) {
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
    <main className={styles.gate}>
      <h1>设置档案密码</h1>
      <p className={styles.muted}>第一次打开需要设置密码。密码至少 8 位。</p>
      <form
        className={styles.form}
        onSubmit={(event) => {
          event.preventDefault();
          if (!busy) {
            void submit();
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
          onChange={(event) => setPassword(event.currentTarget.value)}
        />
        <label htmlFor="create-password-confirm">再输入一次</label>
        <input
          id="create-password-confirm"
          type="password"
          autoComplete="new-password"
          value={passwordConfirm}
          disabled={busy}
          onChange={(event) => setPasswordConfirm(event.currentTarget.value)}
        />
        {error ? <p className={styles.error}>{error}</p> : null}
        <button type="submit" disabled={busy}>
          {busy ? '正在创建…' : '创建档案'}
        </button>
      </form>
    </main>
  );
}
