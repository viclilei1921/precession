import { LockIcon, LockOpenIcon } from '@phosphor-icons/react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { taskEnqueue } from '@/bridge';
import { Button } from '@/components/button';
import { SegmentButton } from '@/components/button/segment';
import { FileDrop } from '@/components/file-drop';
import { PasswordInput } from '@/components/password-input';
import styles from '@/components/record.module.css';
import { Scroll } from '@/components/scroll';
import { TextField } from '@/components/text-field';
import { refreshTasks } from '@/query/task/query';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import { TaskQueue } from './components/task-queue';
import { decryptOutput, encryptOutput } from './path';

type CipherMode = 'encryptFile' | 'decryptFile';

type CipherFile = {
  key: string;
  input: string;
  output: string;
  edited: boolean;
};

let fileKey = 0;

function nextFileKey() {
  fileKey += 1;
  return `cipher-${fileKey}`;
}

export function ToolboxPage() {
  const mobile = useAppStore((state) => state.isMobile());
  if (mobile) {
    return (
      <section className={styles.page}>
        <header className={styles.head}>
          <h1 className={styles.title}>文件加密</h1>
          <p className={styles.hint}>文件加密只在桌面使用。</p>
        </header>
      </section>
    );
  }
  return <FileCipher />;
}

function FileCipher() {
  const queryClient = useQueryClient();
  const [mode, setMode] = useState<CipherMode>('encryptFile');
  const [password, setPassword] = useState('');
  const [files, setFiles] = useState<CipherFile[]>([]);
  const [formError, setFormError] = useState('');

  function addFiles(paths: string[]) {
    const known = new Set(files.map((file) => file.input));
    const added = paths
      .filter((path) => !known.has(path))
      .map((input) => ({
        key: nextFileKey(),
        input,
        output: mode === 'encryptFile' ? encryptOutput(input) : decryptOutput(input),
        edited: false
      }));
    if (added.length === 0) {
      return;
    }
    setFiles((current) => [...current, ...added]);
    setFormError('');
  }

  function switchMode(next: CipherMode) {
    setMode(next);
    setFiles((current) =>
      current.map((file) =>
        file.edited
          ? file
          : { ...file, output: next === 'encryptFile' ? encryptOutput(file.input) : decryptOutput(file.input) }
      )
    );
  }

  const enqueue = useMutation({
    mutationFn: async () => {
      if (files.length === 0) {
        throw new Error('请先添加文件。');
      }
      if (!password) {
        throw new Error('请填写密码。');
      }
      for (const file of files) {
        if (!file.output.trim()) {
          throw new Error('请填写输出路径。');
        }
        await taskEnqueue({ kind: mode, input: file.input, output: file.output.trim(), password });
      }
    },
    onSuccess: async () => {
      setFiles([]);
      setPassword('');
      setFormError('');
      await refreshTasks(queryClient);
    },
    onError: (error) => {
      setFormError(errorMessage(error));
    }
  });

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>文件加密</h1>
        <p className={styles.hint}>拖进窗口或选择文件。加密后文件名加上 .enc，解密时去掉这个后缀。</p>
        <div className={styles.segments} role="tablist" aria-label="加密或解密">
          <SegmentButton selected={mode === 'encryptFile'} onClick={() => switchMode('encryptFile')}>
            <LockIcon weight="regular" />
            加密
          </SegmentButton>
          <SegmentButton selected={mode === 'decryptFile'} onClick={() => switchMode('decryptFile')}>
            <LockOpenIcon weight="regular" />
            解密
          </SegmentButton>
        </div>
        <Button className={styles.trailing} disabled={enqueue.isPending} onClick={() => enqueue.mutate()}>
          加入队列
        </Button>
      </header>
      <Scroll className={styles.scroll}>
        <div className={styles.body}>
          <div className={styles.card}>
            <FileDrop onFiles={addFiles} />
            <PasswordInput
              id="toolbox-file-password"
              label="密码"
              value={password}
              required
              onChange={(event) => setPassword(event.target.value)}
            />
            {formError ? <p className={styles.error}>{formError}</p> : null}
          </div>
          {files.map((file) => (
            <div key={file.key} className={styles.card}>
              <h2 className={styles.cardTitle}>{file.input.split(/[/\\]/).pop()}</h2>
              <TextField
                label="输出路径"
                value={file.output}
                required
                onChange={(event) => {
                  const value = event.target.value;
                  setFiles((current) =>
                    current.map((item) => (item.key === file.key ? { ...item, output: value, edited: true } : item))
                  );
                }}
              />
              <Button
                variant="text"
                tone="destructive"
                onClick={() => setFiles((current) => current.filter((item) => item.key !== file.key))}
              >
                移除
              </Button>
            </div>
          ))}
          <TaskQueue kinds={['encryptFile', 'decryptFile']} />
        </div>
      </Scroll>
    </section>
  );
}
