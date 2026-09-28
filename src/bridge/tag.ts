import { invokeCommand } from './invoke';

/** 标签 */
export type Tag = {
  id: string;
  name: string;
  createdAt: number;
  updatedAt: number;
};

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
