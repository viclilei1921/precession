import { ArrowSquareOutIcon, PencilSimpleIcon, TrashIcon } from '@phosphor-icons/react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { Task, TaskKind } from '@/bridge/task';
import { onTaskQueueUpdated, onTaskUpdate, taskCancel, taskRemove, taskReveal } from '@/bridge/task';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { SegmentButton } from '@/components/button/segment';
import { Progress } from '@/components/progress';
import { usePageSearch } from '@/hooks/search';
import i18n from '@/i18n';
import { applyTaskUpdate, refreshTasks, taskListQuery } from '@/query/task/query';
import { errorMessage } from '@/utils/error';
import { formatRemaining, ProgressETA } from '@/utils/progress-eta';
import { matchesQuery } from '@/utils/search';
import { QualityLine } from './quality-line';
import styles from './queue.module.css';

export type ImageTab = 'run' | 'wait' | 'done';

const kinds: TaskKind[] = ['convertAvif', 'convertJxl'];

type QueueProps = {
  tab: ImageTab;
  onTab: (tab: ImageTab) => void;
  onEdit: (task: Task) => void;
};

type Clock = {
  id: string;
  remaining: number | null;
};

function kindText(task: Task) {
  return i18n.t(task.kind === 'convertJxl' ? 'task:kind.short.jxl' : 'task:kind.short.avif');
}

function sourcePath(task: Task) {
  if (task.draft?.kind === 'convertAvif' || task.draft?.kind === 'convertJxl') {
    return task.draft.input;
  }
  return '';
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

export function ImageQueue({ tab, onTab, onEdit }: QueueProps) {
  const { t } = useTranslation('task');
  const queryClient = useQueryClient();
  const tasksQuery = useQuery(taskListQuery);
  const query = usePageSearch(t('queue.searchPlaceholder'));
  const eta = useRef(new ProgressETA());
  const processingId = useRef<string | null>(null);
  const [clock, setClock] = useState<Clock | null>(null);
  const images = (tasksQuery.data ?? []).filter((task) => kinds.includes(task.kind));
  const visible = images.filter((task) => matches(task, query));
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
      <div className={styles.group} role="tablist" aria-label={t('aria.queue')}>
        <SegmentButton className={styles.tab} selected={tab === 'run'} onClick={() => onTab('run')}>
          {t('queue.tab.run')}
        </SegmentButton>
        <SegmentButton className={styles.tab} selected={tab === 'wait'} onClick={() => onTab('wait')}>
          {t('queue.tab.wait', { count: pending.length })}
        </SegmentButton>
        <SegmentButton className={styles.tab} selected={tab === 'done'} onClick={() => onTab('done')}>
          {t('queue.tab.done', { count: done.length })}
        </SegmentButton>
      </div>
      <div className={styles.panel}>
        {tasksQuery.isPending ? <p className={styles.note}>{t('queue.loading')}</p> : null}
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
                    <span>
                      {clock.remaining == null
                        ? t('time.calculating')
                        : t('time.remaining', { time: formatRemaining(clock.remaining) })}
                    </span>
                  ) : null}
                </div>
                <Button variant="outlined" disabled={cancel.isPending} onClick={() => cancel.mutate(task.id)}>
                  {t('action.cancel')}
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
      <div className={styles.clear}>
        {tab === 'run' ? null : (
          <Button
            variant="text"
            tone="destructive"
            disabled={listed.length === 0 || remove.isPending}
            onClick={() => remove.mutate(listed.map((task) => task.id))}
          >
            {t('queue.clearList')}
          </Button>
        )}
      </div>
    </article>
  );
}

function emptyNote(tab: ImageTab, query: string) {
  if (query.trim()) {
    return i18n.t('task:queue.emptyFiltered');
  }
  if (tab === 'run') {
    return i18n.t('task:queue.emptyRunning');
  }
  if (tab === 'wait') {
    return i18n.t('task:queue.emptyPending');
  }
  return i18n.t('task:queue.emptyDone');
}

type QueueItemProps = {
  task: Task;
  onRemove: () => void;
  removing: boolean;
  onOpen?: () => void;
  onEdit?: () => void;
};

function QueueItem({ task, onRemove, removing, onOpen, onEdit }: QueueItemProps) {
  const { t } = useTranslation('task');
  const path = shownPath(task);
  return (
    <div className={styles.item}>
      <div className={styles.itemMain}>
        <div className={styles.itemName}>
          <span className={styles.title}>{task.title || kindText(task)}</span>
          <span className={styles.kind}>{kindText(task)}</span>
          {task.status === 'failed' ? <span className={styles.kind}>{t('status.failed')}</span> : null}
          {task.status === 'canceled' ? <span className={styles.kind}>{t('status.canceled')}</span> : null}
        </div>
        {path ? (
          <p className={styles.path} title={path}>
            {path}
          </p>
        ) : null}
        {task.quality ? <QualityLine quality={task.quality} /> : null}
        {task.status === 'failed' && task.message ? <p className={styles.detail}>{task.message}</p> : null}
      </div>
      <div className={styles.itemActions}>
        {onOpen ? (
          <IconButton label={t('aria.open')} size="compact" onClick={onOpen}>
            <ArrowSquareOutIcon weight="regular" />
          </IconButton>
        ) : null}
        {onEdit ? (
          <IconButton label={t('action.editAgain')} size="compact" onClick={onEdit}>
            <PencilSimpleIcon weight="regular" />
          </IconButton>
        ) : null}
        <IconButton label={t('action.delete')} tone="destructive" size="compact" disabled={removing} onClick={onRemove}>
          <TrashIcon weight="regular" />
        </IconButton>
      </div>
    </div>
  );
}
