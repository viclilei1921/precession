import type { PlanPatch } from '@/bridge/plan';

/** 只保留这次要改的字段 */
export function toPlanPatch(patch: PlanPatch): PlanPatch {
  return patch;
}
