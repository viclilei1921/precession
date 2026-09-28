import { invokeCommand } from './invoke';

/** 记录归属 */
export type OwnerKind = 'plan' | 'journal_entry' | 'growth_entry' | 'book' | 'book_note';

/** 图片或视频 */
export type MediaKind = 'image' | 'video';

/** 图片或视频元数据 */
export type Media = {
  id: string;
  owner: OwnerKind;
  ownerId: string;
  kind: MediaKind;
  relPath: string;
  mime: string;
  sort: number;
  locked: boolean;
  createdAt: number;
};

/** 写入媒体元数据 */
export type MediaInput = {
  owner: OwnerKind;
  ownerId: string;
  kind: MediaKind;
  relPath: string;
  mime: string;
  sort: number;
  locked: boolean;
};

/** 列出一条记录上的媒体 */
export function mediaList(owner: OwnerKind, ownerId: string) {
  return invokeCommand<Media[]>('media_list', { owner, ownerId });
}

/** 写入媒体元数据 */
export function mediaCreate(input: MediaInput) {
  return invokeCommand<Media>('media_create', { input });
}

/** 删除媒体元数据 */
export function mediaDelete(id: string) {
  return invokeCommand<void>('media_delete', { id });
}
