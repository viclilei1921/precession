import { invokeCommand } from './invoke';

/** 地点 */
export type Place = {
  id: string;
  name: string;
  createdAt: number;
  updatedAt: number;
};

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
