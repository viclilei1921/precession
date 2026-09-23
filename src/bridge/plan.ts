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
export type PlanStepWrite = {
  title: string;
  done: boolean;
};

/** 一条计划 */
export type PlanItem = {
  id: string;
  occurredAt: number;
  title: string;
  body: string;
  locked: boolean;
  highlight: boolean;
  status: PlanStatus;
  priority: number;
  dueAt: number | null;
  result: string;
  steps: PlanStep[];
  createdAt: number;
  updatedAt: number;
};

/** 新建或修改计划 */
export type PlanWrite = {
  occurredAt: number;
  title: string;
  body: string;
  locked: boolean;
  highlight: boolean;
  status: PlanStatus;
  priority: number;
  dueAt: number | null;
  result: string;
  steps: PlanStepWrite[];
};

/** 列出未删除的计划 */
export function planList() {
  return invokeCommand<PlanItem[]>('plan_list');
}

/** 读取一条计划 */
export function planGet(id: string) {
  return invokeCommand<PlanItem>('plan_get', { id });
}

/** 新建计划 */
export function planCreate(input: PlanWrite) {
  return invokeCommand<PlanItem>('plan_create', { input });
}

/** 修改计划 */
export function planUpdate(id: string, input: PlanWrite) {
  return invokeCommand<PlanItem>('plan_update', { id, input });
}

/** 完成计划并记下结果 */
export function planComplete(id: string, result: string) {
  return invokeCommand<PlanItem>('plan_complete', { id, result });
}

/** 软删除计划 */
export function planDelete(id: string) {
  return invokeCommand<void>('plan_delete', { id });
}
