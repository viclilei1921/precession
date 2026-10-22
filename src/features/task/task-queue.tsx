import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useRef, useState } from 'react';
import type { AvifEncodeParams, JxlEncodeParams, Task, TaskKind, VideoEncode } from '@/bridge/task';
import { onTaskQueueUpdated, onTaskUpdate, taskCancel, taskRemove, taskReveal } from '@/bridge/task';
import { Button } from '@/components/button';
import { Card } from '@/components/card';
import { Progress } from '@/components/progress';
import { usePageSearch } from '@/hooks/search';
import { applyTaskUpdate, refreshTasks, taskListQuery } from '@/query/task/query';
import { errorMessage } from '@/utils/error';
import { formatRemaining, ProgressETA } from '@/utils/progress-eta';
import { matchesQuery } from '@/utils/search';
import styles from './task-queue.module.css';

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
  /** 失败或取消后，把这条任务的设置填回当前页。 */
  onEdit?: (task: Task) => void;
};

function videoLine(encode: VideoEncode | undefined) {
  const video = encode?.video;
  const container = encode?.container === 'mov' ? 'MOV' : encode?.container === 'mpegTs' ? 'MPEG-TS' : 'MP4';
  if (video?.codec === 'hevc') {
    const bit = video.bitDepth === 'eight' ? '8-bit' : '10-bit';
    return `${container} · HEVC · ${video.preset ?? 'medium'} · CRF ${video.crf ?? 26} · ${bit}`;
  }
  if (video?.codec === 'vp9') {
    return `${container} · VP9 · 速度 ${video.cpuUsed ?? 2} · CRF ${video.crf ?? 33}`;
  }
  const preset = video?.codec === 'av1' ? (video.preset ?? 6) : 6;
  const crf = video?.codec === 'av1' ? (video.crf ?? 35) : 35;
  const bit = video?.codec === 'av1' && video.bitDepth === 'eight' ? '8-bit' : '10-bit';
  return `${container} · AV1 · 速度 ${preset} · CRF ${crf} · ${bit}`;
}

function avifLine(params: AvifEncodeParams) {
  const quality = params.qcolor ?? 60;
  const speed = params.speed ?? 6;
  const bits = params.depth === 'ten' ? '10-bit' : params.depth === 'twelve' ? '12-bit' : '';
  const loss = params.lossless ? '无损' : '';
  const alpha = params.qalpha != null && params.qalpha !== quality ? `透明 ${params.qalpha}` : '';
  const extra = [bits, loss, alpha].filter(Boolean).join(' · ');
  const base = `质量 ${quality} · 速度 ${speed}`;
  return extra ? `${base} · ${extra}` : base;
}

function jxlLine(params: JxlEncodeParams) {
  const distance = params.quality ?? 1;
  const effort = params.effort ?? 7;
  const loss = params.lossless ? ' · 无损' : '';
  const decoding = params.decodingSpeed ? ` · 解码 ${params.decodingSpeed}` : '';
  const bits = params.overrideBitdepth ? ` · ${params.overrideBitdepth}-bit` : '';
  return `距离 ${distance} · 速度 ${effort}${loss}${decoding}${bits}`;
}

function taskLine(task: Task) {
  const kind = kindLabel[task.kind];
  const draft = task.draft;
  if (!draft) {
    return kind;
  }
  if (draft.kind === 'cutVideo') {
    return `${kind} · ${draft.segments.length} 段 · ${videoLine(draft.encode)}`;
  }
  if (draft.kind === 'convertVideo' || draft.kind === 'mergeVideo' || draft.kind === 'appendVideo') {
    return `${kind} · ${videoLine(draft.encode)}`;
  }
  if (draft.kind === 'convertAvif') {
    return `${kind} · ${avifLine(draft.params)}`;
  }
  if (draft.kind === 'convertJxl') {
    return `${kind} · ${jxlLine(draft.params)}`;
  }
  return kind;
}

type Clock = {
  id: string;
  remaining: number | null;
};

/** 当前页相关的任务。入队和结束刷新整队，编码过程中的进度直接改这一条。 */
export function TaskQueue({ kinds, onEdit }: TaskQueueProps) {
  const queryClient = useQueryClient();
  const tasksQuery = useQuery(taskListQuery);
  const query = usePageSearch('搜索任务');
  const kindKey = kinds.join(',');
  const eta = useRef(new ProgressETA());
  const processingId = useRef<string | null>(null);
  const [clock, setClock] = useState<Clock | null>(null);
  const tasks = (tasksQuery.data ?? []).filter(
    (task) =>
      kinds.includes(task.kind) &&
      matchesQuery(query, kindLabel[task.kind], statusLabel[task.status], task.message, task.title, task.output)
  );

  useEffect(() => {
    const allowed = new Set(kindKey.split(','));
    const current = (tasksQuery.data ?? []).find((task) => allowed.has(task.kind) && task.status === 'processing');
    if (!current) {
      eta.current.reset();
      processingId.current = null;
      setClock(null);
      return;
    }
    if (processingId.current !== current.id) {
      processingId.current = current.id;
      eta.current.reset();
    }
    setClock({ id: current.id, remaining: eta.current.update(current.progress) });
  }, [kindKey, tasksQuery.data]);

  useEffect(() => {
    let stopQueue: (() => void) | undefined;
    let stopTask: (() => void) | undefined;
    let cancelled = false;
    void onTaskQueueUpdated(() => {
      void refreshTasks(queryClient);
    }).then((stop) => {
      if (cancelled) {
        stop();
        return;
      }
      stopQueue = stop;
    });
    void onTaskUpdate((task) => {
      applyTaskUpdate(queryClient, task);
    }).then((stop) => {
      if (cancelled) {
        stop();
        return;
      }
      stopTask = stop;
    });
    return () => {
      cancelled = true;
      stopQueue?.();
      stopTask?.();
    };
  }, [queryClient]);

  const cancel = useMutation({
    mutationFn: (id: string) => taskCancel(id),
    onSuccess: async () => {
      await refreshTasks(queryClient);
    }
  });

  const remove = useMutation({
    mutationFn: async (ids: string[]) => {
      for (const id of ids) {
        await taskRemove(id);
      }
    },
    onSuccess: async () => {
      await refreshTasks(queryClient);
    }
  });

  const removable = (tasksQuery.data ?? []).filter((task) => kinds.includes(task.kind) && task.status !== 'processing');

  const reveal = useMutation({
    mutationFn: (path: string) => taskReveal(path)
  });

  return (
    <div className={styles.queue}>
      <div className={styles.head}>
        <h2 className={styles.heading}>队列</h2>
        <Button
          variant="text"
          tone="destructive"
          disabled={removable.length === 0 || remove.isPending}
          onClick={() => remove.mutate(removable.map((task) => task.id))}
        >
          清除
        </Button>
      </div>
      {tasksQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}
      {!tasksQuery.isPending && tasks.length === 0 ? (
        <Card>
          <p className={styles.note}>{query.trim() ? '没有匹配的任务。' : '现在没有任务。'}</p>
        </Card>
      ) : null}
      {tasks.map((task) => (
        <Card key={task.id} title={task.title || kindLabel[task.kind]} extra={statusLabel[task.status]}>
          <p className={styles.meta}>{taskLine(task)}</p>
          {task.output ? (
            <p className={styles.path} title={task.output}>
              {task.output}
            </p>
          ) : null}
          <Progress value={task.progress} tone="toolbox" label={task.title || kindLabel[task.kind]} />
          <p className={styles.message}>
            <span className={styles.detail}>{task.message || `${Math.round(task.progress)}%`}</span>
            {clock?.id === task.id ? (
              <span className={styles.remain}>
                {clock.remaining == null ? '计算中' : `剩余 ${formatRemaining(clock.remaining)}`}
              </span>
            ) : null}
          </p>
          <div className={styles.actions}>
            {task.status === 'completed' && task.output ? (
              <Button variant="outlined" disabled={reveal.isPending} onClick={() => reveal.mutate(task.output)}>
                显示文件
              </Button>
            ) : null}
            {(task.status === 'failed' || task.status === 'canceled') && task.draft && onEdit ? (
              <Button variant="outlined" onClick={() => onEdit(task)}>
                再次编辑
              </Button>
            ) : null}
            {task.status === 'pending' || task.status === 'processing' ? (
              <Button variant="outlined" disabled={cancel.isPending} onClick={() => cancel.mutate(task.id)}>
                取消
              </Button>
            ) : null}
            {task.status === 'processing' ? null : (
              <Button
                variant="text"
                tone="destructive"
                disabled={remove.isPending}
                onClick={() => remove.mutate([task.id])}
              >
                删除
              </Button>
            )}
          </div>
        </Card>
      ))}
      {tasksQuery.error ? <p className={styles.error}>{errorMessage(tasksQuery.error)}</p> : null}
      {cancel.error ? <p className={styles.error}>{errorMessage(cancel.error)}</p> : null}
      {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
      {reveal.error ? <p className={styles.error}>{errorMessage(reveal.error)}</p> : null}
    </div>
  );
}
