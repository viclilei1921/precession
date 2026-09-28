import { useState } from 'react';
import { errorMessage } from '@/shared/lib/error';
import styles from './session.module.css';
import { useSession } from './store';

export function LockButton() {
  const lock = useSession((state) => state.lock);
  const [locking, setLocking] = useState(false);
  const [error, setError] = useState('');

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
    <div className={styles.lock}>
      {error ? <p className={styles.error}>{error}</p> : null}
      <button type="button" disabled={locking} onClick={() => void lockArchive()}>
        {locking ? '正在锁定…' : '锁定'}
      </button>
    </div>
  );
}
