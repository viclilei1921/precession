import { invokeCommand } from './invoke';

/** 记录类型 */
export type RecordType = 'plan' | 'diary' | 'spark' | 'writing' | 'milestone' | 'moment' | 'excerpt' | 'note';

/** 记录之间的引用 */
export type RecordLinkKind = 'spark_to_writing' | 'writing_to_diary' | 'excerpt_to_journal';

/** 图片或视频 */
export type MediaKind = 'image' | 'video';

/** 今天和回顾看到的卡片 */
export type RecordCard = {
  id: string;
  type: RecordType;
  occurredAt: number;
  title: string;
  body: string;
  subjectMemberId: string | null;
  locked: boolean;
  highlight: boolean;
  createdAt: number;
  updatedAt: number;
};

/** 新建或修改手记、里程碑、精彩瞬间 */
export type RecordWrite = {
  type: 'diary' | 'spark' | 'writing' | 'milestone' | 'moment';
  occurredAt: number;
  title: string;
  body: string;
  subjectMemberId: string | null;
  locked: boolean;
  highlight: boolean;
  memberIds: string[];
  tagIds: string[];
  placeIds: string[];
};

/** 图片或视频元数据 */
export type MediaItem = {
  id: string;
  recordId: string;
  mediaKind: MediaKind;
  relPath: string;
  mime: string;
  sort: number;
  locked: boolean;
  createdAt: number;
};

/** 记录之间的引用 */
export type RecordLink = {
  id: string;
  fromId: string;
  toId: string;
  kind: RecordLinkKind;
  createdAt: number;
};

/** 一条记录及其挂接 */
export type RecordDetail = {
  record: RecordCard;
  memberIds: string[];
  tagIds: string[];
  placeIds: string[];
  media: MediaItem[];
};

/** 按类型和时间列出卡片。from 含，to 不含。 */
export function recordList(recordType?: RecordType, from?: number, to?: number) {
  return invokeCommand<RecordCard[]>('record_list', { recordType, from, to });
}

/** 读取一条记录 */
export function recordGet(id: string) {
  return invokeCommand<RecordDetail>('record_get', { id });
}

/** 新建手记或成长记录 */
export function recordCreate(input: RecordWrite) {
  return invokeCommand<RecordDetail>('record_create', { input });
}

/** 修改手记或成长记录 */
export function recordUpdate(id: string, input: RecordWrite) {
  return invokeCommand<RecordDetail>('record_update', { id, input });
}

/** 软删除手记或成长记录 */
export function recordDelete(id: string) {
  return invokeCommand<void>('record_delete', { id });
}

/** 列出一条记录上的媒体 */
export function mediaList(recordId: string) {
  return invokeCommand<MediaItem[]>('media_list', { recordId });
}

/** 写入媒体元数据 */
export function mediaCreate(
  recordId: string,
  mediaKind: MediaKind,
  relPath: string,
  mime: string,
  sort: number,
  locked: boolean
) {
  return invokeCommand<MediaItem>('media_create', { recordId, mediaKind, relPath, mime, sort, locked });
}

/** 删除媒体元数据 */
export function mediaDelete(id: string) {
  return invokeCommand<void>('media_delete', { id });
}

/** 列出和某条记录相关的引用 */
export function recordLinkList(recordId: string) {
  return invokeCommand<RecordLink[]>('record_link_list', { recordId });
}

/** 建立记录引用 */
export function recordLinkCreate(fromId: string, toId: string, kind: RecordLinkKind) {
  return invokeCommand<RecordLink>('record_link_create', { fromId, toId, kind });
}

/** 删除记录引用 */
export function recordLinkDelete(id: string) {
  return invokeCommand<void>('record_link_delete', { id });
}
