import type { OwnerKind, Task } from '@/bridge';
import {
  onTaskUpdate,
  taskEnqueue,
  taskList,
  uploadAttach,
  uploadDelete,
  uploadDiscard,
  uploadImage,
  uploadList,
  uploadStage,
  uploadTmp
} from '@/bridge';

import { imageIds } from './plain';

const imageExtensions = new Set(['png', 'jpg', 'jpeg', 'gif', 'webp', 'avif']);

const mimeExtension: Record<string, string> = {
  'image/avif': 'avif',
  'image/gif': 'gif',
  'image/jpeg': 'jpg',
  'image/png': 'png',
  'image/webp': 'webp'
};

const urls = new Map<string, string>();

export type MarkdownImageOwner = {
  owner: OwnerKind;
  ownerId: string;
};

/** 记录还没保存时，图片先放在未挂载 */
export const looseImageOwner: MarkdownImageOwner = {
  owner: 'loose',
  ownerId: 'loose'
};

/** 路径的小写图片扩展名。不是图片时返回空 */
export function imageExtension(path: string): string | null {
  const ext = path.split(/[/\\]/).pop()?.split('.').pop()?.toLowerCase() ?? '';
  return imageExtensions.has(ext) ? ext : null;
}

/** 把正文里仍引用的未挂载图片改挂到这条记录上 */
export async function mountLooseImages(owner: OwnerKind, ownerId: string, body: string) {
  const ids = imageIds(body);
  if (ids.length === 0) {
    return;
  }
  await uploadAttach(ids, owner, ownerId);
}

/** 保存后删掉这条记录上、正文里已经不再引用的图片 */
export async function retainBodyImages(owner: OwnerKind, ownerId: string, body: string) {
  const keep = new Set(imageIds(body));
  const rows = await uploadList(owner, ownerId);
  await Promise.all(rows.filter((row) => row.kind === 'image' && !keep.has(row.id)).map((row) => uploadDelete(row.id)));
}

/** 放弃编辑时删掉这次新导入的图片。记录已经不在时忽略 */
export async function discardImages(ids: Iterable<string>) {
  await Promise.all([...ids].map((id) => uploadDelete(id).catch(() => undefined)));
}

/** 把 `upload:` 地址换成可显示的 blob。已加密或找不到时返回占位图 */
export async function uploadDisplayUrl(url: string, owner: MarkdownImageOwner): Promise<string> {
  if (url.startsWith('https://') || url.startsWith('http://')) {
    return url;
  }
  if (!url.startsWith('upload:')) {
    return '';
  }
  const id = url.slice('upload:'.length).trim();
  if (!id) {
    return '';
  }
  const key = `${owner.owner}:${owner.ownerId}:${id}`;
  const cached = urls.get(key);
  if (cached) {
    return cached;
  }
  try {
    const image = await uploadImage(id, owner.owner, owner.ownerId);
    const binary = atob(image.data);
    const bytes = new Uint8Array(binary.length);
    for (let index = 0; index < binary.length; index += 1) {
      bytes[index] = binary.charCodeAt(index);
    }
    const blob = URL.createObjectURL(new Blob([bytes], { type: image.mime || 'image/avif' }));
    urls.set(key, blob);
    return blob;
  } catch (error) {
    const message = error instanceof Error ? error.message : '';
    return message.includes('加密') ? placeholder('图片已加密') : placeholder('图片找不到');
  }
}

/** 从本机路径导入一张图，返回上传文件 id */
export function importImagePath(path: string, owner: MarkdownImageOwner, convert: boolean) {
  const ext = imageExtension(path);
  if (!ext) {
    return Promise.reject(new Error('不支持的文件格式'));
  }
  return importSource(path, ext, owner, convert, []);
}

/** 从剪贴板或拖入的字节导入一张图，返回上传文件 id */
export async function importImageFile(file: File, owner: MarkdownImageOwner, convert: boolean) {
  const ext = imageExtension(file.name) ?? mimeExtension[file.type] ?? null;
  if (!ext) {
    throw new Error('不支持的文件格式');
  }
  const bytes = new Uint8Array(await file.arrayBuffer());
  const staged = await uploadStage(ext, bytesToBase64(bytes));
  return importSource(staged, ext, owner, convert, [staged]);
}

async function importSource(source: string, ext: string, owner: MarkdownImageOwner, convert: boolean, temps: string[]) {
  try {
    let path = source;
    if (convert && ext !== 'avif') {
      const output = await uploadTmp('avif');
      temps.push(output);
      const task = await taskEnqueue({
        kind: 'convertAvif',
        input: source,
        output
      });
      await waitTask(task.id);
      path = output;
    }
    const task = await taskEnqueue({
      kind: 'importUpload',
      source: path,
      owner: owner.owner,
      ownerId: owner.ownerId,
      uploadKind: 'image',
      encrypt: false,
      password: ''
    });
    const done = await waitTask(task.id);
    if (!done.uploadId) {
      throw new Error('导入没有返回文件');
    }
    return done.uploadId;
  } finally {
    await Promise.all(temps.map((path) => uploadDiscard(path).catch(() => undefined)));
  }
}

/** 等到导入或转码任务结束 */
export function waitTask(id: string): Promise<Task> {
  return new Promise((resolve, reject) => {
    let settled = false;
    let stop: (() => void) | undefined;
    const finish = (task: Task) => {
      if (settled || task.id !== id) {
        return;
      }
      if (task.status === 'pending' || task.status === 'processing') {
        return;
      }
      settled = true;
      stop?.();
      if (task.status === 'completed') {
        resolve(task);
        return;
      }
      reject(new Error(task.message || '导入失败'));
    };
    void onTaskUpdate(finish)
      .then((unlisten) => {
        stop = unlisten;
        if (settled) {
          unlisten();
          return;
        }
        return taskList();
      })
      .then((tasks) => {
        const found = tasks?.find((task) => task.id === id);
        if (found) {
          finish(found);
        }
      })
      .catch((error: unknown) => {
        if (!settled) {
          settled = true;
          stop?.();
          reject(error instanceof Error ? error : new Error('导入失败'));
        }
      });
  });
}

function bytesToBase64(bytes: Uint8Array) {
  let binary = '';
  const size = 0x1000;
  for (let index = 0; index < bytes.length; index += size) {
    binary += String.fromCharCode(...bytes.subarray(index, index + size));
  }
  return btoa(binary);
}

function placeholder(label: string) {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="320" height="72"><rect width="100%" height="100%" fill="#f4f4f5"/><text x="16" y="42" font-size="16" fill="#71717a">${label}</text></svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}
