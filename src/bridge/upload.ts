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

/** 读出的一张正文图片。`data` 是标准 base64 */
export type UploadImage = {
  mime: string;
  data: string;
};

/** 读出属于这条记录的未加密图片 */
export function uploadImage(id: string, owner: OwnerKind, ownerId: string) {
  return invokeCommand<UploadImage>('upload_image', { id, owner, ownerId });
}

/** 把粘贴的图片字节暂存到导入临时目录，返回绝对路径 */
export function uploadStage(ext: string, data: string) {
  return invokeCommand<string>('upload_stage', { ext, data });
}

/** 分配一个转码输出用的临时路径，文件本身还不存在 */
export function uploadTmp(ext: string) {
  return invokeCommand<string>('upload_tmp', { ext });
}

/** 删掉导入临时目录里的文件 */
export function uploadDiscard(path: string) {
  return invokeCommand<void>('upload_discard', { path });
}
