import { invokeCommand } from './invoke';
import type { Media } from './media';

/** 计划状态 */
export type PlanStatus = 'inbox' | 'scheduled' | 'done';

/** 计划步骤 */
export type PlanStep = {
  id: string;
  title: string;
  done: boolean;
  sort: number;
};

/** 写入步骤 */
export type PlanStepInput = {
  title: string;
  done: boolean;
};

/** 重复种类 */
export type PlanRepeatKind = 'daily' | 'weekly' | 'monthly' | 'yearly';

/** 重复规则 */
export type PlanRepeat = {
  id: string;
  kind: PlanRepeatKind;
  interval: number;
  weekdays: string;
  untilAt: number | null;
};

/** 写入重复规则 */
export type PlanRepeatInput = {
  kind: PlanRepeatKind;
  interval?: number;
  weekdays?: string;
  untilAt?: number | null;
};

/** 提醒 */
export type PlanReminder = {
  id: string;
  remindAt: number;
  note: string;
  triggered: boolean;
  triggeredAt: number | null;
  createdAt: number;
};

/** 写入提醒 */
export type PlanReminderInput = {
  remindAt: number;
  note?: string;
};

/** 评论 */
export type PlanComment = {
  id: string;
  body: string;
  createdAt: number;
  updatedAt: number;
};

/** 一条计划 */
export type Plan = {
  id: string;
  title: string;
  body: string;
  status: PlanStatus;
  priority: number;
  scheduledAt: number | null;
  dueAt: number | null;
  completedAt: number | null;
  result: string;
  locked: boolean;
  highlight: boolean;
  groupId: string | null;
  parentId: string | null;
  allDay: boolean;
  archived: boolean;
  sort: number;
  timeZone: string;
  steps: PlanStep[];
  memberIds: string[];
  tagIds: string[];
  placeIds: string[];
  repeat: PlanRepeat | null;
  reminders: PlanReminder[];
  media: Media[];
  comments: PlanComment[];
  createdAt: number;
  updatedAt: number;
};

/** 新建计划。标题必填，其余可省略。 */
export type PlanInput = {
  title: string;
  body?: string;
  status?: PlanStatus;
  priority?: number;
  scheduledAt?: number | null;
  dueAt?: number | null;
  result?: string;
  locked?: boolean;
  highlight?: boolean;
  groupId?: string | null;
  parentId?: string | null;
  allDay?: boolean;
  archived?: boolean;
  sort?: number;
  timeZone?: string;
  steps?: PlanStepInput[];
  memberIds?: string[];
  tagIds?: string[];
  placeIds?: string[];
  repeat?: PlanRepeatInput | null;
  reminders?: PlanReminderInput[];
};

/** 更新计划。没传的字段和 null 都保持原值。 */
export type PlanPatch = {
  title?: string;
  body?: string;
  status?: PlanStatus;
  priority?: number;
  scheduledAt?: number;
  dueAt?: number;
  result?: string;
  locked?: boolean;
  highlight?: boolean;
  groupId?: string;
  parentId?: string;
  allDay?: boolean;
  archived?: boolean;
  sort?: number;
  timeZone?: string;
  steps?: PlanStepInput[];
  memberIds?: string[];
  tagIds?: string[];
  placeIds?: string[];
  repeat?: PlanRepeatInput;
  reminders?: PlanReminderInput[];
};

/** 清单 */
export type PlanGroup = {
  id: string;
  name: string;
  color: string;
  system: boolean;
  sort: number;
  createdAt: number;
  updatedAt: number;
};

/** 新建清单 */
export type PlanGroupInput = {
  name: string;
  color?: string;
  system?: boolean;
  sort?: number;
};

/** 更新清单。只改传来的字段。 */
export type PlanGroupPatch = {
  name?: string;
  color?: string;
  sort?: number;
};

/** 列出未删除的计划 */
export function planList() {
  return invokeCommand<Plan[]>('plan_list');
}

/** 读取一条计划 */
export function planGet(id: string) {
  return invokeCommand<Plan>('plan_get', { id });
}

/** 新建计划 */
export function planCreate(input: PlanInput) {
  return invokeCommand<Plan>('plan_create', { input });
}

/** 修改计划。只提交要改的字段。 */
export function planUpdate(id: string, patch: PlanPatch) {
  return invokeCommand<Plan>('plan_update', { id, patch });
}

/** 完成计划并记下结果 */
export function planComplete(id: string, result: string) {
  return invokeCommand<Plan>('plan_complete', { id, result });
}

/** 软删除计划 */
export function planDelete(id: string) {
  return invokeCommand<void>('plan_delete', { id });
}

/** 列出未删除的清单 */
export function planGroupList() {
  return invokeCommand<PlanGroup[]>('plan_group_list');
}

/** 新建清单 */
export function planGroupCreate(input: PlanGroupInput) {
  return invokeCommand<PlanGroup>('plan_group_create', { input });
}

/** 修改清单 */
export function planGroupUpdate(id: string, patch: PlanGroupPatch) {
  return invokeCommand<PlanGroup>('plan_group_update', { id, patch });
}

/** 软删除清单 */
export function planGroupDelete(id: string) {
  return invokeCommand<void>('plan_group_delete', { id });
}

/** 添加评论 */
export function planCommentCreate(planId: string, body: string) {
  return invokeCommand<PlanComment>('plan_comment_create', { planId, body });
}

/** 修改评论 */
export function planCommentUpdate(id: string, body: string) {
  return invokeCommand<PlanComment>('plan_comment_update', { id, body });
}

/** 删除评论 */
export function planCommentDelete(id: string) {
  return invokeCommand<void>('plan_comment_delete', { id });
}
