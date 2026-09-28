import { invokeCommand } from './invoke';

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
  steps: PlanStep[];
  memberIds: string[];
  tagIds: string[];
  placeIds: string[];
  createdAt: number;
  updatedAt: number;
};

/** 新建或修改计划 */
export type PlanInput = {
  title: string;
  body: string;
  status: PlanStatus;
  priority: number;
  scheduledAt: number | null;
  dueAt: number | null;
  result: string;
  locked: boolean;
  highlight: boolean;
  steps: PlanStepInput[];
  memberIds: string[];
  tagIds: string[];
  placeIds: string[];
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

/** 修改计划 */
export function planUpdate(id: string, input: PlanInput) {
  return invokeCommand<Plan>('plan_update', { id, input });
}

/** 完成计划并记下结果 */
export function planComplete(id: string, result: string) {
  return invokeCommand<Plan>('plan_complete', { id, result });
}

/** 软删除计划 */
export function planDelete(id: string) {
  return invokeCommand<void>('plan_delete', { id });
}
