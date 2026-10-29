import { SquaresFourIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import type { Task, TaskKind } from '@/bridge/task';
import { useCardLayout } from '@/features/today/board/board';
import { taskListQuery } from '@/query/task/query';
import { PATH } from '@/router/path';
import { errorMessage } from '@/utils/error';
import styles from './tile.module.css';
import { TileHead } from './tile-head';

const rows: { label: string; to: typeof PATH.cipher | typeof PATH.image | typeof PATH.video; kinds: TaskKind[] }[] = [
  { label: '文件加密', to: PATH.cipher, kinds: ['encryptFile', 'decryptFile'] },
  { label: '图片转码', to: PATH.image, kinds: ['convertAvif', 'convertJxl'] },
  { label: '视频处理', to: PATH.video, kinds: ['convertVideo', 'cutVideo', 'mergeVideo', 'appendVideo'] }
];

function runningCount(tasks: Task[], kinds: TaskKind[]): number {
  return tasks.filter(
    (task) => kinds.includes(task.kind) && (task.status === 'pending' || task.status === 'processing')
  ).length;
}

export function ToolboxCard() {
  const navigate = useNavigate();
  const tasksQuery = useQuery(taskListQuery);
  const tasks = tasksQuery.data ?? [];
  const counts = rows.map((row) => runningCount(tasks, row.kinds));

  useCardLayout('toolbox', `${tasksQuery.isPending}:${counts.join(',')}`);

  return (
    <article className={styles.card}>
      <TileHead tone="toolbox" icon={SquaresFourIcon} title="工具箱" />
      {tasksQuery.error ? <p className={styles.error}>{errorMessage(tasksQuery.error)}</p> : null}
      <ul className={styles.counts}>
        {rows.map((row, index) => (
          <li key={row.label}>
            <button type="button" onClick={() => void navigate({ to: row.to })}>
              <strong>{counts[index]}</strong>
              <span>{row.label}</span>
            </button>
          </li>
        ))}
      </ul>
    </article>
  );
}
