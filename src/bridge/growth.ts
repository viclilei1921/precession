import { invokeCommand } from './invoke';

/** 成长记录种类 */
export type GrowthKind = 'milestone' | 'moment';

/** 一条成长记录 */
export type GrowthEntry = {
  id: string;
  kind: GrowthKind;
  memberId: string;
  occurredAt: number;
  title: string;
  body: string;
  locked: boolean;
  highlight: boolean;
  memberIds: string[];
  tagIds: string[];
  placeIds: string[];
  createdAt: number;
  updatedAt: number;
};

/** 新建或修改成长记录 */
export type GrowthEntryInput = {
  kind: GrowthKind;
  memberId: string;
  occurredAt: number;
  title: string;
  body: string;
  locked: boolean;
  highlight: boolean;
  memberIds: string[];
  tagIds: string[];
  placeIds: string[];
};

/** 按成员、种类和时间列出成长记录。from 含，to 不含。 */
export function growthEntryList(memberId?: string, kind?: GrowthKind, from?: number, to?: number) {
  return invokeCommand<GrowthEntry[]>('growth_entry_list', { memberId, kind, from, to });
}

/** 读取一条成长记录 */
export function growthEntryGet(id: string) {
  return invokeCommand<GrowthEntry>('growth_entry_get', { id });
}

/** 新建成长记录 */
export function growthEntryCreate(input: GrowthEntryInput) {
  return invokeCommand<GrowthEntry>('growth_entry_create', { input });
}

/** 修改成长记录 */
export function growthEntryUpdate(id: string, input: GrowthEntryInput) {
  return invokeCommand<GrowthEntry>('growth_entry_update', { id, input });
}

/** 软删除成长记录 */
export function growthEntryDelete(id: string) {
  return invokeCommand<void>('growth_entry_delete', { id });
}
