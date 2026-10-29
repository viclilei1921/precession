import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import type { Task, UploadKind } from '@/bridge';
import { onTaskUpdate, taskEnqueue, taskList } from '@/bridge';
import { Button } from '@/components/button';
import { Modal } from '@/components/modal';
import { PasswordInput } from '@/components/password-input';
import { Select } from '@/components/select';
import { TextField } from '@/components/text-field';
import { refreshFiles } from '@/query/files/query';
import { errorMessage } from '@/utils/error';
import styles from './page.module.css';
import { LOOSE_OWNER } from './text';

const kinds: { id: UploadKind; label: string }[] = [
  { id: 'image', label: '图片' },
  { id: 'video', label: '视频' },
  { id: 'book', label: '书籍' },
  { id: 'file', label: '其他' }
];

export type ImportDraft = {
  path: string;
  name: string;
  kind: UploadKind;
  sealed: boolean;
};

function waitTask(id: string) {
  return new Promise<Task>((resolve, reject) => {
    let settled = false;
    let stop: (() => void) | undefined;
    const finish = (task: Task) => {
      if (settled || task.id !== id || task.status === 'pending' || task.status === 'processing') {
        return;
      }
      settled = true;
      stop?.();
      if (task.status === 'completed') {
        resolve(task);
        return;
      }
      reject(new Error(task.message || '导入失败'));
    };
    void onTaskUpdate(finish)
      .then((unlisten) => {
        stop = unlisten;
        if (settled) {
          unlisten();
          return;
        }
        return taskList();
      })
      .then((tasks) => {
        const found = tasks?.find((task) => task.id === id);
        if (found) {
          finish(found);
        }
      })
      .catch((error: unknown) => {
        if (!settled) {
          settled = true;
          stop?.();
          reject(error instanceof Error ? error : new Error('导入失败'));
        }
      });
  });
}

export function ImportDialog({
  files,
  onClose,
  onImported
}: {
  files: ImportDraft[];
  onClose: () => void;
  onImported: (id: string | null) => void;
}) {
  const queryClient = useQueryClient();
  const [drafts, setDrafts] = useState(files);
  const [password, setPassword] = useState('');
  const sealedCount = drafts.filter((file) => file.sealed).length;
  const allSealed = drafts.length > 0 && sealedCount === drafts.length;
  const save = useMutation({
    mutationFn: async () => {
      const ids: string[] = [];
      for (const file of drafts) {
        const name = file.name.trim();
        if (!name) {
          throw new Error('请填写文件名。');
        }
        const task = await taskEnqueue({
          kind: 'importUpload',
          source: file.path,
          owner: LOOSE_OWNER,
          ownerId: LOOSE_OWNER,
          uploadKind: file.kind,
          name,
          encrypt: !file.sealed && password.length > 0,
          password: file.sealed ? '' : password
        });
        const done = await waitTask(task.id);
        if (done.uploadId) {
          ids.push(done.uploadId);
        }
      }
      return ids.at(-1) ?? null;
    },
    onSuccess: async (id) => {
      await refreshFiles(queryClient);
      onImported(id);
    },
    onError: async () => {
      await refreshFiles(queryClient);
    }
  });

  return (
    <Modal title="上传" wide onClose={onClose}>
      <div className={styles.importList}>
        {drafts.map((file, index) => (
          <div key={file.path} className={styles.importRow}>
            <TextField
              label="名称"
              value={file.sealed || password.length > 0 ? `${file.name}.env` : file.name}
              onChange={(event) => {
                const raw = event.target.value;
                const marking = file.sealed || password.length > 0;
                const name = marking && raw.toLowerCase().endsWith('.env') ? raw.slice(0, -4) : raw;
                setDrafts((current) =>
                  current.map((item, itemIndex) => (itemIndex === index ? { ...item, name } : item))
                );
              }}
            />
            <Select
              label="类型"
              value={file.kind}
              onChange={(event) => {
                const kind = event.target.value as UploadKind;
                setDrafts((current) =>
                  current.map((item, itemIndex) => (itemIndex === index ? { ...item, kind } : item))
                );
              }}
            >
              {kinds.map((item) => (
                <option key={item.id} value={item.id}>
                  {item.label}
                </option>
              ))}
            </Select>
          </div>
        ))}
      </div>
      {sealedCount > 0 ? <p className={styles.importNote}>已经加密的文件会原样入库，并标成加密文件。</p> : null}
      {allSealed ? null : (
        <PasswordInput
          label="密码"
          value={password}
          placeholder={sealedCount > 0 ? '只加密还没加密的文件' : '留空则不加密，填写后名称加上 .env'}
          onChange={(event) => setPassword(event.target.value)}
        />
      )}
      {save.error ? <p className={styles.error}>{errorMessage(save.error)}</p> : null}
      <div className={styles.dialogActions}>
        <Button variant="text" onClick={onClose}>
          取消
        </Button>
        <Button disabled={save.isPending || drafts.length === 0} onClick={() => save.mutate()}>
          {save.isPending ? '正在导入…' : '导入'}
        </Button>
      </div>
    </Modal>
  );
}
