import { LockIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import type { UploadCatalog } from '@/bridge/upload';
import { mediaSrc, uploadOpen } from '@/bridge/upload';
import { Button } from '@/components/button';
import { PasswordInput } from '@/components/password-input';
import { mediaInfoQuery } from '@/query/files/query';
import { errorMessage } from '@/utils/error';
import styles from './access.module.css';

type Phase = 'check' | 'need' | 'ready' | 'error';

/** 这次会话里已经解开的文件。再次打开不用再要一次密码。 */
const opened = new Set<string>();

export type MediaAccess = {
  playable: boolean;
  phase: Phase;
  message: string;
  password: string;
  setPassword: (value: string) => void;
  opening: boolean;
  unlock: () => Promise<void>;
  src: string;
  pending: boolean;
};

function canPlay(file: UploadCatalog) {
  return (file.kind === 'image' || file.kind === 'video') && !file.ownerSealed;
}

/** 向本机预览服务要播放地址。图片和视频共用。 */
export function useMediaAccess(file: UploadCatalog): MediaAccess {
  const playable = canPlay(file);
  const endpoint = useQuery({ ...mediaInfoQuery, enabled: playable });
  const [fileId, setFileId] = useState(file.id);
  const [phase, setPhase] = useState<Phase>(() => (opened.has(file.id) ? 'ready' : 'check'));
  const [message, setMessage] = useState('');
  const [password, setPassword] = useState('');
  const [opening, setOpening] = useState(false);

  if (file.id !== fileId) {
    setFileId(file.id);
    setPhase(opened.has(file.id) ? 'ready' : 'check');
    setMessage('');
    setPassword('');
    setOpening(false);
  }

  useEffect(() => {
    if (!playable) {
      return;
    }
    if (opened.has(file.id)) {
      setPhase('ready');
      return;
    }
    let cancel = false;
    setPhase('check');
    setMessage('');
    uploadOpen(file.id, '')
      .then(() => {
        opened.add(file.id);
        if (!cancel) {
          setPhase('ready');
        }
      })
      .catch((error: unknown) => {
        if (cancel) {
          return;
        }
        const text = errorMessage(error);
        if (text === '请输入密码') {
          setPhase('need');
          return;
        }
        setMessage(text);
        setPhase('error');
      });
    return () => {
      cancel = true;
    };
  }, [file.id, playable]);

  async function unlock() {
    setOpening(true);
    setMessage('');
    try {
      await uploadOpen(file.id, password);
      opened.add(file.id);
      setPassword('');
      setPhase('ready');
    } catch (error) {
      setMessage(errorMessage(error));
    } finally {
      setOpening(false);
    }
  }

  const src = phase === 'ready' && endpoint.data ? mediaSrc(endpoint.data, file.id) : '';

  return {
    playable,
    phase,
    message,
    password,
    setPassword,
    opening,
    unlock,
    src,
    pending: endpoint.isPending
  };
}

/** 加密文件的密码。解开后由调用方显示画面。 */
export function UnlockForm({ access }: { access: MediaAccess }) {
  return (
    <form
      className={styles.form}
      onSubmit={(event) => {
        event.preventDefault();
        void access.unlock();
      }}
    >
      <LockIcon className={styles.lock} weight="regular" />
      <span>输入密码后可以预览</span>
      <PasswordInput
        label="密码"
        value={access.password}
        autoFocus
        onChange={(event) => access.setPassword(event.target.value)}
      />
      {access.message ? <span className={styles.fail}>{access.message}</span> : null}
      <Button type="submit" disabled={access.opening || access.password.length === 0}>
        {access.opening ? '正在打开…' : '打开'}
      </Button>
    </form>
  );
}
