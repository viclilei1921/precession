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
export type MemberWrite = {
  name: string;
  relation: string;
  gender: string;
  birthday: number | null;
};

/** 标签 */
export type Tag = {
  id: string;
  name: string;
  createdAt: number;
  updatedAt: number;
};

/** 地点 */
export type Place = {
  id: string;
  name: string;
  createdAt: number;
  updatedAt: number;
};

/** 列出未删除的成员 */
export function memberList() {
  return invokeCommand<Member[]>('member_list');
}

/** 按 id 读取成员 */
export function memberGet(id: string) {
  return invokeCommand<Member>('member_get', { id });
}

/** 新建成员 */
export function memberCreate(input: MemberWrite) {
  return invokeCommand<Member>('member_create', { input });
}

/** 修改成员 */
export function memberUpdate(id: string, input: MemberWrite) {
  return invokeCommand<Member>('member_update', { id, input });
}

/** 软删除成员 */
export function memberDelete(id: string) {
  return invokeCommand<void>('member_delete', { id });
}

/** 列出未删除的标签 */
export function tagList() {
  return invokeCommand<Tag[]>('tag_list');
}

/** 新建标签 */
export function tagCreate(name: string) {
  return invokeCommand<Tag>('tag_create', { name });
}

/** 修改标签 */
export function tagUpdate(id: string, name: string) {
  return invokeCommand<Tag>('tag_update', { id, name });
}

/** 软删除标签 */
export function tagDelete(id: string) {
  return invokeCommand<void>('tag_delete', { id });
}

/** 列出未删除的地点 */
export function placeList() {
  return invokeCommand<Place[]>('place_list');
}

/** 新建地点 */
export function placeCreate(name: string) {
  return invokeCommand<Place>('place_create', { name });
}

/** 修改地点 */
export function placeUpdate(id: string, name: string) {
  return invokeCommand<Place>('place_update', { id, name });
}

/** 软删除地点 */
export function placeDelete(id: string) {
  return invokeCommand<void>('place_delete', { id });
}
