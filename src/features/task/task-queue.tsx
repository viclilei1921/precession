import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { AvifEncodeParams, JxlEncodeParams, Task, TaskKind, VideoEncode } from '@/bridge/task';
import { onTaskQueueUpdated, onTaskUpdate, taskCancel, taskRemove, taskReveal } from '@/bridge/task';
import { Button } from '@/components/button';
import { Card } from '@/components/card';
import { Progress } from '@/components/progress';
import { QualityLine } from '@/features/image/components/quality-line';
import { usePageSearch } from '@/hooks/search';
import i18n from '@/i18n';
import { applyTaskUpdate, refreshTasks, taskListQuery } from '@/query/task/query';
import { errorMessage } from '@/utils/error';
import { formatRemaining, ProgressETA } from '@/utils/progress-eta';
import { matchesQuery } from '@/utils/search';
import styles from './task-queue.module.css';

const kindKeys: Record<TaskKind, string> = {
  encryptFile: 'task:kind.encryptFile',
  decryptFile: 'task:kind.decryptFile',
  importUpload: 'task:kind.importUpload',
  encryptUpload: 'task:kind.encryptUpload',
  decryptUpload: 'task:kind.decryptUpload',
  convertVideo: 'task:kind.convertVideo',
  cutVideo: 'task:kind.cutVideo',
  mergeVideo: 'task:kind.mergeVideo',
  appendVideo: 'task:kind.appendVideo',
  convertAvif: 'task:kind.convertAvif',
  convertJxl: 'task:kind.convertJxl'
};

const statusKeys: Record<Task['status'], string> = {
  pending: 'task:status.pending',
  processing: 'task:status.processing',
  completed: 'task:status.completed',
  failed: 'task:status.failed',
  canceled: 'task:status.canceled'
};

function kindLabel(kind: TaskKind) {
  return i18n.t(kindKeys[kind]);
}

function statusLabel(status: Task['status']) {
  return i18n.t(statusKeys[status]);
}

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
    return i18n.t('task:video.hevc', {
      container,
      preset: video.preset ?? 'medium',
      crf: video.crf ?? 26,
      bit
    });
  }
  if (video?.codec === 'vp9') {
    return i18n.t('task:video.vp9', {
      container,
      speed: video.cpuUsed ?? 2,
      crf: video.crf ?? 33
    });
  }
  const preset = video?.codec === 'av1' ? (video.preset ?? 6) : 6;
  const crf = video?.codec === 'av1' ? (video.crf ?? 35) : 35;
  const bit = video?.codec === 'av1' && video.bitDepth === 'eight' ? '8-bit' : '10-bit';
  return i18n.t('task:video.av1', { container, speed: preset, crf, bit });
}

function avifLine(params: AvifEncodeParams) {
  const quality = params.qcolor ?? 60;
  const speed = params.speed ?? 6;
  const bits = params.depth === 'ten' ? '10-bit' : params.depth === 'twelve' ? '12-bit' : '';
  const loss = params.lossless ? i18n.t('task:avif.lossless') : '';
  const alpha =
    params.qalpha != null && params.qalpha !== quality ? i18n.t('task:avif.alpha', { value: params.qalpha }) : '';
  const extra = [bits, loss, alpha].filter(Boolean).join(' · ');
  const base = i18n.t('task:avif.summary', { quality, speed });
  return extra ? `${base} · ${extra}` : base;
}

function jxlLine(params: JxlEncodeParams) {
  const distance = params.quality ?? 1;
  const effort = params.effort ?? 7;
  const loss = params.lossless ? ` · ${i18n.t('task:jxl.lossless')}` : '';
  const decoding = params.decodingSpeed ? ` · ${i18n.t('task:jxl.decoding', { speed: params.decodingSpeed })}` : '';
  const bits = params.overrideBitdepth ? ` · ${i18n.t('task:bitDepth', { depth: params.overrideBitdepth })}` : '';
  return `${i18n.t('task:jxl.summary', { distance, effort })}${loss}${decoding}${bits}`;
}

function taskLine(task: Task) {
  const kind = kindLabel(task.kind);
  const draft = task.draft;
  if (!draft) {
    return kind;
  }
  if (draft.kind === 'cutVideo') {
    return `${kind} · ${i18n.t('task:count.segments', { count: draft.segments.length })} · ${videoLine(draft.encode)}`;
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
  const { t } = useTranslation('task');
  const queryClient = useQueryClient();
  const tasksQuery = useQuery(taskListQuery);
  const query = usePageSearch(t('queue.searchPlaceholder'));
  const kindKey = kinds.join(',');
  const eta = useRef(new ProgressETA());
  const processingId = useRef<string | null>(null);
  const [clock, setClock] = useState<Clock | null>(null);
  const tasks = (tasksQuery.data ?? []).filter(
    (task) =>
      kinds.includes(task.kind) &&
      matchesQuery(query, kindLabel(task.kind), statusLabel(task.status), task.message, task.title, task.output)
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
        <h2 className={styles.heading}>{t('queue.heading')}</h2>
        <Button
          variant="text"
          tone="destructive"
          disabled={removable.length === 0 || remove.isPending}
          onClick={() => remove.mutate(removable.map((task) => task.id))}
        >
          {t('queue.clearList')}
        </Button>
      </div>
      {tasksQuery.isPending ? <p className={styles.note}>{t('queue.loading')}</p> : null}
      {!tasksQuery.isPending && tasks.length === 0 ? (
        <Card>
          <p className={styles.note}>{query.trim() ? t('queue.emptyFiltered') : t('queue.empty')}</p>
        </Card>
      ) : null}
      {tasks.map((task) => (
        <Card key={task.id} title={task.title || kindLabel(task.kind)} extra={statusLabel(task.status)}>
          <p className={styles.meta}>{taskLine(task)}</p>
          {task.output ? (
            <p className={styles.path} title={task.output}>
              {task.output}
            </p>
          ) : null}
          {task.quality ? <QualityLine quality={task.quality} /> : null}
          <Progress value={task.progress} tone="toolbox" label={task.title || kindLabel(task.kind)} />
          <p className={styles.message}>
            <span className={styles.detail}>{task.message || `${Math.round(task.progress)}%`}</span>
            {clock?.id === task.id ? (
              <span className={styles.remain}>
                {clock.remaining == null
                  ? t('time.calculating')
                  : t('time.remaining', { time: formatRemaining(clock.remaining) })}
              </span>
            ) : null}
          </p>
          <div className={styles.actions}>
            {task.status === 'completed' && task.output ? (
              <Button variant="outlined" disabled={reveal.isPending} onClick={() => reveal.mutate(task.output)}>
                {t('action.reveal')}
              </Button>
            ) : null}
            {(task.status === 'failed' || task.status === 'canceled') && task.draft && onEdit ? (
              <Button variant="outlined" onClick={() => onEdit(task)}>
                {t('action.editAgain')}
              </Button>
            ) : null}
            {task.status === 'pending' || task.status === 'processing' ? (
              <Button variant="outlined" disabled={cancel.isPending} onClick={() => cancel.mutate(task.id)}>
                {t('action.cancel')}
              </Button>
            ) : null}
            {task.status === 'processing' ? null : (
              <Button
                variant="text"
                tone="destructive"
                disabled={remove.isPending}
                onClick={() => remove.mutate([task.id])}
              >
                {t('action.delete')}
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
