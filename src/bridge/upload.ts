import { invokeCommand } from './invoke';

/** 记录归属 */
export type OwnerKind = 'plan' | 'journal_entry' | 'growth_entry' | 'book' | 'book_note' | 'member' | 'loose';

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

/** 把未挂载的文件改挂到一条已存在的记录上。已经挂在别处的标识会跳过 */
export function uploadAttach(ids: string[], owner: OwnerKind, ownerId: string) {
  return invokeCommand<void>('upload_attach', { ids, owner, ownerId });
}

/** 把文件的明文写到用户选的文件夹，返回写好的绝对路径。加密文件在还没解开时需要密码 */
export function uploadExport(id: string, dir: string, password: string) {
  return invokeCommand<string>('upload_export', { id, dir, password });
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

/** 文件目录里的一条。所属记录已不在时标题为空，书摘才有父书 */
export type UploadCatalog = Upload & {
  ownerTitle: string;
  /** 手记、成长、书摘的种类，或书的书架状态。计划为空 */
  ownerKind: string;
  ownerSealed: boolean;
  /** 手记、成长、书摘的发生时间，书和计划的创建时间。记录已不在时为 0 */
  ownerAt: number;
  parentId: string;
  parentTitle: string;
};

/** 一次列出全部文件，并带上所属记录 */
export function uploadCatalog() {
  return invokeCommand<UploadCatalog[]>('upload_catalog');
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

/** 本机播放地址。`origin` 为空表示预览服务没有起来 */
export type MediaEndpoint = {
  origin: string;
  token: string;
};

/** 准备预览。未加密或这次已经解开过时，密码可以是空字符串 */
export function uploadOpen(id: string, password: string) {
  return invokeCommand<void>('upload_open', { id, password });
}

/** 读取本机播放地址 */
export function mediaInfo() {
  return invokeCommand<MediaEndpoint>('media_info');
}

/** 拼出一份文件的播放地址 */
export function mediaSrc(endpoint: MediaEndpoint, id: string) {
  if (!endpoint.origin) {
    return '';
  }
  return `${endpoint.origin}/m/${endpoint.token}/${id}`;
}
