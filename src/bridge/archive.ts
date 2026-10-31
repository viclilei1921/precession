import { open, save } from '@tauri-apps/plugin-dialog';
import { invokeCommand } from './invoke';

/** 当前档案的位置和上传文件用量 */
export type ArchiveInfo = {
  path: string;
  fileCount: number;
  fileBytes: number;
};

/** 导出完成的压缩包 */
export type ArchiveExport = {
  path: string;
  fileCount: number;
  bytes: number;
};

const ARCHIVE_FILTER = [{ name: 'Precession 档案', extensions: ['zip'] }];

/** 当天的档案文件名，例如 precession-2026-10-31.zip */
export function todayArchiveName() {
  const now = new Date();
  const month = String(now.getMonth() + 1).padStart(2, '0');
  const day = String(now.getDate()).padStart(2, '0');
  return `precession-${now.getFullYear()}-${month}-${day}.zip`;
}

/** 读取数据库路径，以及上传目录里正式文件的个数和字节数 */
export function archiveInfo() {
  return invokeCommand<ArchiveInfo>('archive_info');
}

/** 把数据库和上传文件写到指定压缩包 */
export function archiveExport(path: string) {
  return invokeCommand<ArchiveExport>('archive_export', { path });
}

/** 写到应用缓存并打开系统分享。name 只是文件名 */
export function archiveExportShare(name: string) {
  return invokeCommand<ArchiveExport>('archive_export_share', { name });
}

/** 用压缩包替换当前数据库和上传文件。成功后会话已关闭 */
export function archiveImport(path: string) {
  return invokeCommand<void>('archive_import', { path });
}

/** 让用户选择压缩包的保存位置。取消时返回空字符串 */
export async function pickArchiveSave() {
  const selected = await save({
    defaultPath: todayArchiveName(),
    filters: ARCHIVE_FILTER
  });
  return selected ?? '';
}

/** 让用户选择要导入的压缩包。取消时返回空字符串 */
export async function pickArchiveOpen() {
  const selected = await open({
    multiple: false,
    directory: false,
    fileAccessMode: 'scoped',
    filters: ARCHIVE_FILTER
  });
  return typeof selected === 'string' ? selected : '';
}
