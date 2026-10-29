import { invokeCommand } from './invoke';

/** 时间轴上的记录种类 */
export type TimelineKind = 'plan' | 'diary' | 'spark' | 'writing' | 'milestone' | 'moment' | 'excerpt' | 'note';

/** 今天和回顾看到的一条记录 */
export type TimelineItem = {
  id: string;
  kind: TimelineKind;
  occurredAt: number;
  title: string;
  body: string;
  locked: boolean;
  highlight: boolean;
};

/** 按发生时间聚合。from 含，to 不含。 */
export function timelineList(from?: number, to?: number, kinds?: TimelineKind[]) {
  return invokeCommand<TimelineItem[]>('timeline_list', { from, to, kinds });
}
