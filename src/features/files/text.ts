import type { OwnerKind, UploadCatalog, UploadKind } from '@/bridge/upload';
import { matchesQuery } from '@/utils/search';

export type FileModule = 'journal' | 'library' | 'plan' | 'growth' | 'loose';

/** 不挂在记录上的文件。归属和归属 ID 都用这个值 */
export const LOOSE_OWNER = 'loose';

export type KindFilter = 'all' | UploadKind;

export type StateFilter = 'all' | 'preview' | 'encrypted' | 'sealed';

export type FileSort = 'recent' | 'size' | 'name';

export type FileFilters = {
  module: FileModule | 'all';
  kind: KindFilter;
  state: StateFilter;
  query: string;
};

export const FILE_MODULES: { id: FileModule; label: string }[] = [
  { id: 'journal', label: '手记' },
  { id: 'library', label: '书库' },
  { id: 'plan', label: '计划' },
  { id: 'growth', label: '成长' },
  { id: 'loose', label: '未挂载' }
];

export const KIND_FILTERS: { id: KindFilter; label: string }[] = [
  { id: 'all', label: '全部' },
  { id: 'image', label: '图片' },
  { id: 'book', label: '书籍' },
  { id: 'video', label: '视频' },
  { id: 'file', label: '其他' }
];

const kindLabels: Record<string, string> = {
  diary: '日记',
  spark: '灵感',
  writing: '写作',
  milestone: '里程碑',
  moment: '瞬间',
  want: '想读',
  reading: '在读',
  finished: '读完',
  excerpt: '书摘',
  note: '笔记',
  avatar: '头像'
};

export function moduleOf(owner: OwnerKind): FileModule {
  if (owner === 'loose') {
    return 'loose';
  }
  if (owner === 'journal_entry') {
    return 'journal';
  }
  if (owner === 'book' || owner === 'book_note') {
    return 'library';
  }
  if (owner === 'plan') {
    return 'plan';
  }
  if (owner === 'member') {
    return 'growth';
  }
  return 'growth';
}

export function moduleLabel(module: FileModule) {
  return FILE_MODULES.find((item) => item.id === module)?.label ?? module;
}

export function kindLabel(file: UploadCatalog) {
  if (file.owner === 'plan') {
    return '计划';
  }
  return kindLabels[file.ownerKind] ?? '';
}

export function ownerKey(file: UploadCatalog) {
  return `${file.owner}:${file.ownerId}`;
}

export function recordTitle(file: UploadCatalog) {
  if (file.owner === 'loose') {
    return '未挂载';
  }
  return file.ownerTitle.trim() || '记录已不在';
}

export function formatBytes(bytes: number) {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${Math.round(bytes / 1024)} KB`;
  }
  const mb = bytes / (1024 * 1024);
  return mb >= 10 ? `${mb.toFixed(0)} MB` : `${mb.toFixed(1)} MB`;
}

/** 文件扩展名。末尾是 `.env` 或 `.enc` 且前面还有一层时，用前面那层 */
function nameExt(name: string) {
  const parts = name.split('.');
  if (parts.length < 2) {
    return '';
  }
  const last = parts.pop()?.toLowerCase() ?? '';
  if ((last === 'env' || last === 'enc') && parts.length >= 2) {
    return parts.pop()?.toLowerCase() ?? '';
  }
  return last;
}

/** 封面角标。azw 和 azw3 按 MOBI 显示 */
export function bookFormatLabel(name: string) {
  const ext = nameExt(name);
  if (ext === 'epub') {
    return 'EPUB';
  }
  if (ext === 'txt') {
    return 'TXT';
  }
  if (ext === 'mobi' || ext === 'azw' || ext === 'azw3') {
    return 'MOBI';
  }
  return ext ? ext.toUpperCase() : '书籍';
}

function sameRecord(files: UploadCatalog[], file: UploadCatalog) {
  const key = ownerKey(file);
  return files.filter((item) => ownerKey(item) === key);
}

function bookFiles(files: UploadCatalog[]) {
  return files
    .filter((item) => item.kind === 'book')
    .sort((left, right) => left.sort - right.sort || left.createdAt - right.createdAt);
}

/** 同一本书上，倒序后第一份未加密的书籍文件 */
export function readableBook(files: UploadCatalog[], file: UploadCatalog) {
  return [...bookFiles(sameRecord(files, file))].reverse().find((item) => !item.encrypted);
}

export function canPreview(file: UploadCatalog) {
  if (file.ownerSealed) {
    return false;
  }
  return file.kind === 'image' || file.kind === 'video';
}

/** 列表缩略图只拉未加密、未封存的图片 */
export function canThumb(file: UploadCatalog) {
  return file.kind === 'image' && !file.encrypted && !file.ownerSealed;
}

export function displayName(file: UploadCatalog, files: UploadCatalog[]) {
  if (file.owner === 'member') {
    return '头像';
  }
  if (file.owner === 'loose' || file.kind !== 'image') {
    return file.name || '未命名文件';
  }
  const images = sameRecord(files, file)
    .filter((item) => item.kind === 'image')
    .sort((left, right) => left.sort - right.sort || left.createdAt - right.createdAt);
  if (images.length <= 1) {
    return '正文图片';
  }
  const index = images.findIndex((item) => item.id === file.id) + 1;
  return `正文图片 ${index}`;
}

export function fileRole(file: UploadCatalog, files: UploadCatalog[]) {
  if (file.kind !== 'book') {
    return '';
  }
  const current = readableBook(files, file);
  if (current?.id === file.id) {
    return '当前正文';
  }
  if (file.encrypted) {
    return '已加密';
  }
  return '较早的一份';
}

export function readingNote(file: UploadCatalog, files: UploadCatalog[]) {
  if (file.owner !== 'book') {
    return '';
  }
  const books = bookFiles(sameRecord(files, file));
  if (books.length === 0) {
    return '没有正文';
  }
  if (readableBook(files, file)) {
    return '可阅读';
  }
  if (books.every((item) => item.encrypted)) {
    return '已加密，不能阅读';
  }
  return '没有可打开的正文';
}

function matchesState(file: UploadCatalog, state: StateFilter) {
  if (state === 'preview') {
    return canPreview(file);
  }
  if (state === 'encrypted') {
    return file.encrypted;
  }
  if (state === 'sealed') {
    return file.ownerSealed;
  }
  return true;
}

export function matchesFile(
  file: UploadCatalog,
  files: UploadCatalog[],
  filters: FileFilters,
  ignore?: 'kind' | 'state'
) {
  if (filters.module !== 'all' && moduleOf(file.owner) !== filters.module) {
    return false;
  }
  if (ignore !== 'kind' && filters.kind !== 'all' && file.kind !== filters.kind) {
    return false;
  }
  if (ignore !== 'state' && !matchesState(file, filters.state)) {
    return false;
  }
  return matchesQuery(
    filters.query,
    displayName(file, files),
    file.name,
    file.ownerTitle,
    kindLabel(file),
    file.parentTitle,
    file.kind === 'book' ? bookFormatLabel(file.name) : ''
  );
}

export function countForKind(files: UploadCatalog[], filters: FileFilters, kind: KindFilter) {
  return files.filter((file) => {
    if (kind !== 'all' && file.kind !== kind) {
      return false;
    }
    return matchesFile(file, files, filters, 'kind');
  }).length;
}

export function compareFiles(left: UploadCatalog, right: UploadCatalog, sort: FileSort, files: UploadCatalog[]) {
  if (sort === 'size') {
    return right.size - left.size;
  }
  if (sort === 'name') {
    return displayName(left, files).localeCompare(displayName(right, files), 'zh');
  }
  return right.createdAt - left.createdAt;
}

export function openLabel(file: UploadCatalog) {
  if (file.owner === 'book') {
    return '打开这本书';
  }
  if (file.owner === 'book_note') {
    return '打开这条书摘';
  }
  if (file.owner === 'plan') {
    return '打开这条计划';
  }
  if (file.owner === 'growth_entry') {
    return '打开这条成长';
  }
  if (file.owner === 'member') {
    return '打开这位家人';
  }
  return '打开这篇手记';
}

export function canOpen(file: UploadCatalog) {
  if (file.owner === 'loose') {
    return false;
  }
  if (!file.ownerTitle.trim()) {
    return false;
  }
  if (file.owner === 'book_note') {
    return file.parentId.length > 0;
  }
  return true;
}

type DeleteCopy = {
  blocked: boolean;
  title: string;
  body: string;
};

function hostName(file: UploadCatalog) {
  if (file.owner === 'book_note') {
    return '这条书摘';
  }
  if (file.owner === 'plan') {
    return '这条计划';
  }
  if (file.owner === 'growth_entry') {
    return '这条成长记录';
  }
  if (file.owner === 'member') {
    return '这位家人';
  }
  return '这篇手记';
}

const IMAGE_EXT = new Set(['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'avif', 'heic', 'heif', 'jxl', 'tif', 'tiff']);
const VIDEO_EXT = new Set(['mp4', 'm4v', 'mov', 'mkv', 'webm', 'avi', 'ts', 'm2ts', 'mpeg', 'mpg', 'wmv']);
const BOOK_EXT = new Set(['epub', 'pdf', 'mobi', 'azw', 'azw3', 'txt']);

/** 按扩展名猜文件类型，猜不到就是其他 */
export function kindFromName(name: string): UploadKind {
  const ext = nameExt(name);
  if (IMAGE_EXT.has(ext)) {
    return 'image';
  }
  if (VIDEO_EXT.has(ext)) {
    return 'video';
  }
  if (BOOK_EXT.has(ext)) {
    return 'book';
  }
  return 'file';
}

/** 工具箱加密结果以 `.enc` 结尾 */
export function isSealedFileName(name: string) {
  return name.toLowerCase().endsWith('.enc');
}

/** 去掉末尾的 `.enc`。不是这个后缀时原样返回 */
export function plainFileName(name: string) {
  return isSealedFileName(name) ? name.slice(0, -4) : name;
}

/** 从路径取出文件名 */
export function fileBaseName(path: string) {
  const name = path.split(/[/\\]/).pop();
  return name && name.length > 0 ? name : path;
}

export function deleteCopy(file: UploadCatalog, files: UploadCatalog[]): DeleteCopy {
  if (file.owner === 'loose') {
    return {
      blocked: false,
      title: '删除这份文件',
      body: '它不挂在记录上。删掉后就没有了。'
    };
  }
  const title = file.ownerTitle.trim();
  const quoted = title ? `《${title}》` : '这条记录';
  if (file.ownerSealed) {
    return {
      blocked: true,
      title: '还不能删除',
      body: `${quoted}已封存。正文里仍引用这个文件，先解开再删。`
    };
  }
  if (file.kind === 'book') {
    const owned = sameRecord(files, file);
    const rest = owned.filter((item) => item.id !== file.id);
    const current = readableBook(files, file);
    const after = [...bookFiles(rest)].reverse().find((item) => !item.encrypted);
    if (current?.id === file.id && !after) {
      const encryptedLeft = rest.some((item) => item.kind === 'book' && item.encrypted);
      return {
        blocked: false,
        title: '删除这份正文',
        body: encryptedLeft
          ? `删除后，${quoted}只剩已加密的文件。书还在书架上，阅读页会显示暂时不能阅读。进度和封面保留。`
          : `删除后，${quoted}还在书架上，但没有可打开的正文。进度和封面保留。`
      };
    }
    if (current && current.id !== file.id) {
      return {
        blocked: false,
        title: '删除这份文件',
        body: `${quoted}仍用 ${bookFormatLabel(current.name)} 打开，阅读不受影响。`
      };
    }
    return {
      blocked: false,
      title: '删除这份正文',
      body: `删除后，${quoted}没有正文文件。书还在书架上。`
    };
  }
  if (file.owner === 'member') {
    const name = file.ownerTitle.trim();
    return {
      blocked: false,
      title: '删除这张头像',
      body: name ? `删掉后，${name}的头像会空着。家人还在。` : '删掉后，这位家人的头像会空着。家人还在。'
    };
  }
  if (file.kind === 'image') {
    return {
      blocked: false,
      title: '删除这张正文图片',
      body: `它会从${hostName(file)}${title ? quoted : ''}里去掉。记录本身保留。`
    };
  }
  return {
    blocked: false,
    title: '删除这份文件',
    body: `它会从${hostName(file)}${title ? quoted : ''}里去掉。记录本身保留。`
  };
}
