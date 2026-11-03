import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { fileSizes, taskEnqueue } from '@/bridge';
import type { Task } from '@/bridge/task';
import { Button } from '@/components/button';
import { SegmentButton } from '@/components/button/segment';
import { FileDrop } from '@/components/file-drop';
import { PasswordInput } from '@/components/password-input';
import { InfoTip } from '@/components/tooltip';
import { refreshTasks } from '@/query/task/query';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import type { CipherFile } from './components/file-list';
import { FileList } from './components/file-list';
import type { CipherTab } from './components/queue';
import { CipherQueue } from './components/queue';
import styles from './page.module.css';
import { decryptOutput, encryptOutput } from './path';

type CipherMode = 'encryptFile' | 'decryptFile';

let fileKey = 0;

function nextFileKey() {
  fileKey += 1;
  return `cipher-${fileKey}`;
}

export function CipherPage() {
  const mobile = useAppStore((state) => state.isMobile());
  if (mobile) {
    return <section className={styles.page} />;
  }
  return <CipherWorkspace />;
}

function CipherWorkspace() {
  const { t } = useTranslation('cipher');
  const queryClient = useQueryClient();
  const [mode, setMode] = useState<CipherMode>('encryptFile');
  const [password, setPassword] = useState('');
  const [files, setFiles] = useState<CipherFile[]>([]);
  const [formError, setFormError] = useState('');
  const [tab, setTab] = useState<CipherTab>('run');

  const enqueue = useMutation({
    mutationFn: async () => {
      const selected = files.filter((file) => file.checked);
      if (files.length === 0) {
        throw new Error(t('error.noFiles'));
      }
      if (selected.length === 0) {
        throw new Error(t('error.noSelection'));
      }
      if (!password) {
        throw new Error(t('error.passwordRequired'));
      }
      for (const file of selected) {
        const output = mode === 'encryptFile' ? encryptOutput(file.input) : decryptOutput(file.input);
        await taskEnqueue({
          kind: mode,
          input: file.input,
          output,
          password
        });
      }
    },
    onSuccess: async () => {
      setFiles((current) => current.filter((file) => !file.checked));
      setPassword('');
      setFormError('');
      setTab('wait');
      await refreshTasks(queryClient);
    },
    onError: (error) => {
      setFormError(errorMessage(error));
    }
  });

  async function addFiles(paths: string[]) {
    const incoming = paths.filter((path) => path.length > 0);
    if (incoming.length === 0) {
      return;
    }
    const sizes = new Map<string, number>();
    try {
      const found = await fileSizes(incoming);
      for (const item of found) {
        sizes.set(item.path, item.size);
      }
    } catch {
      sizes.clear();
    }
    setFiles((current) => {
      const known = new Set(current.map((file) => file.input));
      const next = incoming
        .filter((path) => !known.has(path))
        .map((input) => ({
          key: nextFileKey(),
          input,
          size: sizes.get(input) ?? null,
          checked: true
        }));
      return next.length === 0 ? current : [...current, ...next];
    });
    setFormError('');
  }

  function restore(task: Task) {
    if (task.draft?.kind !== 'encryptFile' && task.draft?.kind !== 'decryptFile') {
      return;
    }
    const draft = task.draft;
    setMode(draft.kind);
    setFiles([
      {
        key: nextFileKey(),
        input: draft.input,
        size: null,
        checked: true
      }
    ]);
    setFormError('');
    void fileSizes([draft.input])
      .then((found) => {
        const size = found.find((item) => item.path === draft.input)?.size ?? null;
        setFiles((current) => current.map((file) => (file.input === draft.input ? { ...file, size } : file)));
      })
      .catch(() => undefined);
  }

  return (
    <section className={styles.page}>
      <article className={styles.toolbar}>
        <div className={styles.tools}>
          <fieldset className={styles.group} aria-label={t('aria.modeGroup')}>
            <SegmentButton selected={mode === 'encryptFile'} onClick={() => setMode('encryptFile')}>
              {t('mode.encrypt')}
            </SegmentButton>
            <SegmentButton selected={mode === 'decryptFile'} onClick={() => setMode('decryptFile')}>
              {t('mode.decrypt')}
            </SegmentButton>
          </fieldset>
          <InfoTip label={t('page.hintLabel')} title={t('page.hint')} />
        </div>
        <PasswordInput
          className={styles.password}
          label={t('page.password')}
          required
          value={password}
          autoComplete="off"
          onChange={(event) => {
            setPassword(event.target.value);
            setFormError('');
          }}
        />
        <div className={styles.algo}>
          <span className={styles.algoLabel}>{t('page.algorithm')}</span>
          <span className={styles.algoValue}>ChaCha20</span>
        </div>
      </article>
      <div className={styles.workspace}>
        <article className={styles.fileCard}>
          <div className={styles.fileTools}>
            <FileDrop onFiles={(paths) => void addFiles(paths)} />
            <div className={styles.toolsEnd}>
              {formError ? <p className={styles.error}>{formError}</p> : null}
              <Button disabled={enqueue.isPending} onClick={() => enqueue.mutate()}>
                {mode === 'encryptFile' ? t('page.startEncrypt') : t('page.startDecrypt')}
              </Button>
            </div>
          </div>
          <FileList
            files={files}
            onChecked={(key, checked) => {
              setFiles((current) => current.map((file) => (file.key === key ? { ...file, checked } : file)));
            }}
            onRemove={(key) => {
              setFiles((current) => current.filter((file) => file.key !== key));
            }}
            onSelectAll={() => {
              setFiles((current) => current.map((file) => ({ ...file, checked: true })));
            }}
            onInvert={() => {
              setFiles((current) => current.map((file) => ({ ...file, checked: !file.checked })));
            }}
            onClear={() => setFiles([])}
            onDeleteSelected={() => {
              setFiles((current) => current.filter((file) => !file.checked));
            }}
          />
        </article>
        <CipherQueue tab={tab} onTab={setTab} onEdit={restore} />
      </div>
    </section>
  );
}
