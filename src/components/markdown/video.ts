import type { OwnerKind } from '@/bridge';
import { taskEnqueue, uploadAttach, uploadDelete, uploadList } from '@/bridge';

import type { MarkdownImageOwner } from './image';
import { waitTask } from './image';
import { videoIds } from './plain';

const videoExtensions = new Set(['mp4', 'm4v', 'mov', 'mkv', 'webm', 'ts', 'm2ts', 'mpeg', 'mpg']);

export const videoExts = [...videoExtensions];

/** 路径的小写视频扩展名。不是视频时返回空 */
export function videoExtension(path: string): string | null {
  const ext = path.split(/[/\\]/).pop()?.split('.').pop()?.toLowerCase() ?? '';
  return videoExtensions.has(ext) ? ext : null;
}

/** 从本机路径导入一段视频，返回上传文件 id。不转码 */
export async function importVideoPath(path: string, owner: MarkdownImageOwner) {
  if (!videoExtension(path)) {
    throw new Error('不支持的文件格式');
  }
  const task = await taskEnqueue({
    kind: 'importUpload',
    source: path,
    owner: owner.owner,
    ownerId: owner.ownerId,
    uploadKind: 'video',
    encrypt: false,
    password: ''
  });
  const done = await waitTask(task.id);
  if (!done.uploadId) {
    throw new Error('导入没有返回文件');
  }
  return done.uploadId;
}

/** 把正文里仍引用的未挂载视频改挂到这条记录上 */
export async function mountLooseVideos(owner: OwnerKind, ownerId: string, body: string) {
  const ids = videoIds(body);
  if (ids.length === 0) {
    return;
  }
  await uploadAttach(ids, owner, ownerId);
}

/** 保存后删掉这条记录上、正文里已经不再引用的视频 */
export async function retainBodyVideos(owner: OwnerKind, ownerId: string, body: string) {
  const keep = new Set(videoIds(body));
  const rows = await uploadList(owner, ownerId);
  await Promise.all(rows.filter((row) => row.kind === 'video' && !keep.has(row.id)).map((row) => uploadDelete(row.id)));
}
