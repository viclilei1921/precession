import type { Plan, PlanInput } from '@/bridge/plan';

/** 用现有计划拼写入参数，再盖上要改的字段 */
export function toPlanInput(plan: Plan, patch: Partial<PlanInput> = {}): PlanInput {
  return {
    title: plan.title,
    body: plan.body,
    status: plan.status,
    priority: plan.priority,
    scheduledAt: plan.scheduledAt,
    dueAt: plan.dueAt,
    result: plan.result,
    locked: plan.locked,
    highlight: plan.highlight,
    steps: plan.steps.map((step) => ({ title: step.title, done: step.done })),
    memberIds: plan.memberIds,
    tagIds: plan.tagIds,
    placeIds: plan.placeIds,
    ...patch
  };
}
