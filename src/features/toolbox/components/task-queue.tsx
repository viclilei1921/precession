import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';
import type { Task, TaskKind } from '@/bridge/task';
import { onTaskQueueUpdated, taskCancel } from '@/bridge/task';
import { Button } from '@/components/button';
import styles from '@/components/record.module.css';
import { usePageSearch } from '@/hooks/search';
import { refreshTasks, taskListQuery } from '@/query/task/query';
import { errorMessage } from '@/utils/error';
import { matchesQuery } from '@/utils/search';

const kindLabel: Record<TaskKind, string> = {
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

type TaskQueueProps = {
  kinds: TaskKind[];
};

/** 当前页相关的任务队列。进度来自队列事件。 */
export function TaskQueue({ kinds }: TaskQueueProps) {
  const queryClient = useQueryClient();
  const tasksQuery = useQuery(taskListQuery);
  const query = usePageSearch('搜索任务');
  const tasks = (tasksQuery.data ?? []).filter(
    (task) =>
      kinds.includes(task.kind) && matchesQuery(query, kindLabel[task.kind], statusLabel[task.status], task.message)
  );

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    void onTaskQueueUpdated(() => {
      void refreshTasks(queryClient);
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

  const cancel = useMutation({
    mutationFn: (id: string) => taskCancel(id),
    onSuccess: async () => {
      await refreshTasks(queryClient);
    }
  });

  return (
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
  );
}
