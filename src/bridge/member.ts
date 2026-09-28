import { invokeCommand } from './invoke';

/** 成员档案 */
export type Member = {
  id: string;
  name: string;
  relation: string;
  gender: string;
  birthday: number | null;
  createdAt: number;
  updatedAt: number;
};

/** 新建或修改成员 */
export type MemberInput = {
  name: string;
  relation: string;
  gender: string;
  birthday: number | null;
};

/** 列出未删除的成员 */
export function memberList() {
  return invokeCommand<Member[]>('member_list');
}

/** 读取一个成员 */
export function memberGet(id: string) {
  return invokeCommand<Member>('member_get', { id });
}

/** 新建成员 */
export function memberCreate(input: MemberInput) {
  return invokeCommand<Member>('member_create', { input });
}

/** 修改成员 */
export function memberUpdate(id: string, input: MemberInput) {
  return invokeCommand<Member>('member_update', { id, input });
}

/** 软删除成员 */
export function memberDelete(id: string) {
  return invokeCommand<void>('member_delete', { id });
}
