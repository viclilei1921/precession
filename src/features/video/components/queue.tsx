import { ArrowSquareOutIcon, PencilSimpleIcon, TrashIcon } from '@phosphor-icons/react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useRef, useState } from 'react';
import type { Task, TaskKind } from '@/bridge/task';
import { onTaskQueueUpdated, onTaskUpdate, taskCancel, taskRemove, taskReveal } from '@/bridge/task';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { SegmentButton } from '@/components/button/segment';
import { Progress } from '@/components/progress';
import { Scroll } from '@/components/scroll';
import { usePageSearch } from '@/hooks/search';
import { applyTaskUpdate, refreshTasks, taskListQuery } from '@/query/task/query';
import { errorMessage } from '@/utils/error';
import { formatRemaining, ProgressETA } from '@/utils/progress-eta';
import { matchesQuery } from '@/utils/search';
import styles from './queue.module.css';

export type VideoTab = 'run' | 'wait' | 'done';

const kinds: TaskKind[] = ['convertVideo', 'cutVideo', 'mergeVideo', 'appendVideo'];

const kindLabel: Partial<Record<TaskKind, string>> = {
  convertVideo: '转码',
  cutVideo: '裁剪',
  mergeVideo: '合并',
  appendVideo: '追加'
};

type QueueProps = {
  tab: VideoTab;
  onTab: (tab: VideoTab) => void;
  onEdit: (task: Task) => void;
};

type Clock = {
  id: string;
  remaining: number | null;
};

function kindText(task: Task) {
  return kindLabel[task.kind] ?? task.kind;
}

function sourcePath(task: Task) {
  const draft = task.draft;
  if (!draft) {
    return '';
  }
  switch (draft.kind) {
    case 'convertVideo':
    case 'cutVideo':
      return draft.input;
    case 'mergeVideo':
      return draft.inputs[0] ?? '';
    case 'appendVideo':
      return draft.base;
    default:
      return '';
  }
}

function shownPath(task: Task) {
  if (task.status === 'completed' && task.output) {
    return task.output;
  }
  return sourcePath(task) || task.output;
}

function matches(task: Task, query: string) {
  return matchesQuery(query, kindText(task), task.title, task.message, task.output, sourcePath(task));
}

export function VideoQueue({ tab, onTab, onEdit }: QueueProps) {
  const queryClient = useQueryClient();
  const tasksQuery = useQuery(taskListQuery);
  const query = usePageSearch('搜索任务');
  const eta = useRef(new ProgressETA());
  const processingId = useRef<string | null>(null);
  const [clock, setClock] = useState<Clock | null>(null);
  const videos = (tasksQuery.data ?? []).filter((task) => kinds.includes(task.kind));
  const visible = videos.filter((task) => matches(task, query));
  const running = visible.filter((task) => task.status === 'processing');
  const pending = visible.filter((task) => task.status === 'pending');
  const done = visible.filter(
    (task) => task.status === 'completed' || task.status === 'failed' || task.status === 'canceled'
  );

  useEffect(() => {
    const current = (tasksQuery.data ?? []).find((task) => kinds.includes(task.kind) && task.status === 'processing');
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
  }, [tasksQuery.data]);

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

  const reveal = useMutation({
    mutationFn: (path: string) => taskReveal(path)
  });

  const listed = tab === 'run' ? running : tab === 'wait' ? pending : done;
  const error = tasksQuery.error ?? cancel.error ?? remove.error ?? reveal.error;

  return (
    <article className={styles.queue}>
      <div className={styles.group} role="tablist" aria-label="队列">
        <SegmentButton className={styles.tab} selected={tab === 'run'} onClick={() => onTab('run')}>
          进行中
        </SegmentButton>
        <SegmentButton className={styles.tab} selected={tab === 'wait'} onClick={() => onTab('wait')}>
          待处理 {pending.length}
        </SegmentButton>
        <SegmentButton className={styles.tab} selected={tab === 'done'} onClick={() => onTab('done')}>
          已完成 {done.length}
        </SegmentButton>
      </div>
      <Scroll className={styles.panel} axis="vertical" gutter>
        <div className={styles.list}>
          {tasksQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}
          {!tasksQuery.isPending && listed.length === 0 ? <p className={styles.note}>{emptyNote(tab, query)}</p> : null}
          {tab === 'run'
            ? running.map((task) => (
                <div className={styles.run} key={task.id}>
                  <div className={styles.itemName}>
                    <span className={styles.title}>{task.title || kindText(task)}</span>
                    <span className={styles.kind}>{kindText(task)}</span>
                  </div>
                  <p className={styles.path} title={shownPath(task)}>
                    {shownPath(task)}
                  </p>
                  <Progress value={task.progress} tone="toolbox" label={task.title || kindText(task)} />
                  <div className={styles.meta}>
                    <span>{task.message || `${Math.round(task.progress)}%`}</span>
                    {clock?.id === task.id ? (
                      <span>{clock.remaining == null ? '计算中' : `剩余 ${formatRemaining(clock.remaining)}`}</span>
                    ) : null}
                  </div>
                  <Button variant="outlined" disabled={cancel.isPending} onClick={() => cancel.mutate(task.id)}>
                    取消
                  </Button>
                </div>
              ))
            : null}
          {tab === 'wait'
            ? pending.map((task) => (
                <QueueItem
                  key={task.id}
                  task={task}
                  onRemove={() => remove.mutate([task.id])}
                  removing={remove.isPending}
                />
              ))
            : null}
          {tab === 'done'
            ? done.map((task) => (
                <QueueItem
                  key={task.id}
                  task={task}
                  onRemove={() => remove.mutate([task.id])}
                  removing={remove.isPending}
                  onOpen={task.status === 'completed' && task.output ? () => reveal.mutate(task.output) : undefined}
                  onEdit={
                    (task.status === 'failed' || task.status === 'canceled') && task.draft
                      ? () => onEdit(task)
                      : undefined
                  }
                />
              ))
            : null}
          {error ? <p className={styles.error}>{errorMessage(error)}</p> : null}
        </div>
      </Scroll>
      <div className={styles.clear}>
        {tab === 'run' ? null : (
          <Button
            variant="text"
            tone="destructive"
            disabled={listed.length === 0 || remove.isPending}
            onClick={() => remove.mutate(listed.map((task) => task.id))}
          >
            清空列表
          </Button>
        )}
      </div>
    </article>
  );
}

function emptyNote(tab: VideoTab, query: string) {
  if (query.trim()) {
    return '没有匹配的任务。';
  }
  if (tab === 'run') {
    return '没有正在处理的任务。';
  }
  if (tab === 'wait') {
    return '没有待处理的任务。';
  }
  return '还没有完成的任务。';
}

type QueueItemProps = {
  task: Task;
  onRemove: () => void;
  removing: boolean;
  onOpen?: () => void;
  onEdit?: () => void;
};

function QueueItem({ task, onRemove, removing, onOpen, onEdit }: QueueItemProps) {
  const path = shownPath(task);
  return (
    <div className={styles.item}>
      <div className={styles.itemMain}>
        <div className={styles.itemName}>
          <span className={styles.title}>{task.title || kindText(task)}</span>
          <span className={styles.kind}>{kindText(task)}</span>
          {task.status === 'failed' ? <span className={styles.kind}>失败</span> : null}
          {task.status === 'canceled' ? <span className={styles.kind}>已取消</span> : null}
        </div>
        {path ? (
          <p className={styles.path} title={path}>
            {path}
          </p>
        ) : null}
        {task.status === 'failed' && task.message ? <p className={styles.detail}>{task.message}</p> : null}
      </div>
      <div className={styles.itemActions}>
        {onOpen ? (
          <IconButton label="打开" size="compact" onClick={onOpen}>
            <ArrowSquareOutIcon weight="regular" />
          </IconButton>
        ) : null}
        {onEdit ? (
          <IconButton label="再次编辑" size="compact" onClick={onEdit}>
            <PencilSimpleIcon weight="regular" />
          </IconButton>
        ) : null}
        <IconButton label="删除" tone="destructive" size="compact" disabled={removing} onClick={onRemove}>
          <TrashIcon weight="regular" />
        </IconButton>
      </div>
    </div>
  );
}
