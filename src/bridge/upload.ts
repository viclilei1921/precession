import { invokeCommand } from './invoke';

/** 记录归属 */
export type OwnerKind = 'plan' | 'journal_entry' | 'growth_entry' | 'book' | 'book_note';

/** 图片、视频、书或其他文件 */
export type UploadKind = 'image' | 'video' | 'book' | 'file';

/** 上传到应用里的文件。本体在数据目录的 `upload/{kind}/` 下 */
export type Upload = {
  id: string;
  owner: OwnerKind;
  ownerId: string;
  kind: UploadKind;
  relPath: string;
  mime: string;
  /** 原始文件名 */
  name: string;
  /** 导入时的字节数 */
  size: number;
  sort: number;
  locked: boolean;
  encrypted: boolean;
  createdAt: number;
};

/** 写入文件元数据 */
export type UploadInput = {
  owner: OwnerKind;
  ownerId: string;
  kind: UploadKind;
  relPath: string;
  mime: string;
  name?: string;
  size?: number;
  sort: number;
  locked: boolean;
};

/** 列出一条记录上的文件 */
export function uploadList(owner: OwnerKind, ownerId: string) {
  return invokeCommand<Upload[]>('upload_list', { owner, ownerId });
}

/** 写入文件元数据 */
export function uploadCreate(input: UploadInput) {
  return invokeCommand<Upload>('upload_create', { input });
}

/** 删除文件记录和文件 */
export function uploadDelete(id: string) {
  return invokeCommand<void>('upload_delete', { id });
}
