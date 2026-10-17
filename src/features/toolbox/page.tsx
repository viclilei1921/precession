import { LockIcon, LockOpenIcon } from '@phosphor-icons/react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import type { Task } from '@/bridge/task';
import { onTaskQueueUpdated, taskCancel, taskEnqueue } from '@/bridge/task';
import { Button } from '@/components/button';
import { SegmentButton } from '@/components/button/segment';
import { PasswordInput } from '@/components/password-input';
import styles from '@/components/record.module.css';
import { TextField } from '@/components/text-field';
import { usePageSearch } from '@/hooks/search';
import { taskListQuery } from '@/query/task/query';
import { errorMessage } from '@/utils/error';
import { matchesQuery } from '@/utils/search';

const kindLabel: Record<Task['kind'], string> = {
  encryptFile: '加密文件',
  decryptFile: '解密文件',
  importMedia: '导入影像',
  encryptMedia: '加密影像',
  decryptMedia: '解密影像',
  convertVideo: '转码视频',
  cutVideo: '裁剪视频',
  mergeVideo: '合并视频',
  appendVideo: '追加视频',
  convertAvif: '转为 AVIF',
  convertJxl: '转为 JXL'
};

const statusLabel: Record<Task['status'], string> = {
  pending: '等待',
  processing: '处理中',
  completed: '完成',
  failed: '失败',
  canceled: '已取消'
};

export function ToolboxPage() {
  const queryClient = useQueryClient();
  const tasksQuery = useQuery(taskListQuery);
  const query = usePageSearch('搜索任务');
  const tasks = (tasksQuery.data ?? []).filter((task) =>
    matchesQuery(query, kindLabel[task.kind], statusLabel[task.status], task.message)
  );
  const [mode, setMode] = useState<'encryptFile' | 'decryptFile'>('encryptFile');
  const [input, setInput] = useState('');
  const [output, setOutput] = useState('');
  const [password, setPassword] = useState('');

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    void onTaskQueueUpdated(() => {
      void queryClient.invalidateQueries({ queryKey: ['task', 'list'] });
    }).then((stop) => {
      if (cancelled) {
        stop();
        return;
      }
      unlisten = stop;
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [queryClient]);

  const enqueue = useMutation({
    mutationFn: () => taskEnqueue({ kind: mode, input: input.trim(), output: output.trim(), password }),
    onSuccess: async () => {
      setPassword('');
      await queryClient.invalidateQueries({ queryKey: ['task', 'list'] });
    }
  });
  const cancel = useMutation({
    mutationFn: (id: string) => taskCancel(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['task', 'list'] });
    }
  });

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>工具箱</h1>
        <p className={styles.hint}>文件加解密在这里提交。转码、裁剪、合并和图片转码走同一条队列。</p>
      </header>
      <form
        className={styles.card}
        onSubmit={(event) => {
          event.preventDefault();
          enqueue.mutate();
        }}
      >
        <h2 className={styles.cardTitle}>加密或解密一个文件</h2>
        <div className={styles.segments}>
          <SegmentButton selected={mode === 'encryptFile'} onClick={() => setMode('encryptFile')}>
            <LockIcon weight="regular" />
            加密
          </SegmentButton>
          <SegmentButton selected={mode === 'decryptFile'} onClick={() => setMode('decryptFile')}>
            <LockOpenIcon weight="regular" />
            解密
          </SegmentButton>
        </div>
        <TextField label="输入路径" value={input} required onChange={(event) => setInput(event.target.value)} />
        <TextField label="输出路径" value={output} required onChange={(event) => setOutput(event.target.value)} />
        <PasswordInput
          id="toolbox-file-password"
          label="密码"
          value={password}
          required
          onChange={(event) => setPassword(event.target.value)}
        />
        {enqueue.error ? <p className={styles.error}>{errorMessage(enqueue.error)}</p> : null}
        <Button type="submit" className={styles.trailing} disabled={enqueue.isPending}>
          加入队列
        </Button>
      </form>
      <div className={styles.card}>
        <h2 className={styles.cardTitle}>队列</h2>
        {tasksQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}
        {!tasksQuery.isPending && tasks.length === 0 ? (
          <p className={styles.empty}>{query.trim() ? '没有匹配的任务。' : '现在没有任务。'}</p>
        ) : null}
        {tasks.map((task) => (
          <div key={task.id} className={styles.row}>
            <span className={styles.rowTitle}>
              {kindLabel[task.kind]} · {statusLabel[task.status]} · {Math.round(task.progress)}%
            </span>
            <span className={styles.meta}>{task.message}</span>
            {task.status === 'pending' || task.status === 'processing' ? (
              <Button variant="outlined" disabled={cancel.isPending} onClick={() => cancel.mutate(task.id)}>
                取消
              </Button>
            ) : null}
          </div>
        ))}
        {tasksQuery.error ? <p className={styles.error}>{errorMessage(tasksQuery.error)}</p> : null}
        {cancel.error ? <p className={styles.error}>{errorMessage(cancel.error)}</p> : null}
      </div>
    </section>
  );
}
