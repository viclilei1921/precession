import type { OwnerKind, UploadCatalog, UploadKind } from '@/bridge/upload';
import i18n, { currentLocale } from '@/i18n';
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

export const FILE_MODULES: { id: FileModule; labelKey: string }[] = [
  { id: 'journal', labelKey: 'types.modules.journal' },
  { id: 'library', labelKey: 'types.modules.library' },
  { id: 'plan', labelKey: 'types.modules.plan' },
  { id: 'growth', labelKey: 'types.modules.growth' },
  { id: 'loose', labelKey: 'types.modules.loose' }
];

export const KIND_FILTERS: { id: KindFilter; labelKey: string }[] = [
  { id: 'all', labelKey: 'types.filters.all' },
  { id: 'image', labelKey: 'types.filters.image' },
  { id: 'book', labelKey: 'types.filters.book' },
  { id: 'video', labelKey: 'types.filters.video' },
  { id: 'file', labelKey: 'types.filters.file' }
];

const kindLabels: Record<string, string> = {
  diary: 'types.ownerKinds.diary',
  spark: 'types.ownerKinds.spark',
  writing: 'types.ownerKinds.writing',
  milestone: 'types.ownerKinds.milestone',
  moment: 'types.ownerKinds.moment',
  want: 'types.ownerKinds.want',
  reading: 'types.ownerKinds.reading',
  finished: 'types.ownerKinds.finished',
  excerpt: 'types.ownerKinds.excerpt',
  note: 'types.ownerKinds.note',
  avatar: 'types.ownerKinds.avatar'
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
  const item = FILE_MODULES.find((entry) => entry.id === module);
  return item ? i18n.t(`files:${item.labelKey}`) : module;
}

export function kindFilterLabel(kind: KindFilter) {
  const item = KIND_FILTERS.find((entry) => entry.id === kind);
  return item ? i18n.t(`files:${item.labelKey}`) : kind;
}

export function kindLabel(file: UploadCatalog) {
  if (file.owner === 'plan') {
    return i18n.t('files:types.modules.plan');
  }
  const key = kindLabels[file.ownerKind];
  return key ? i18n.t(`files:${key}`) : '';
}

export function ownerKey(file: UploadCatalog) {
  return `${file.owner}:${file.ownerId}`;
}

export function recordTitle(file: UploadCatalog) {
  if (file.owner === 'loose') {
    return i18n.t('files:types.modules.loose');
  }
  return file.ownerTitle.trim() || i18n.t('files:record.missing');
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
  return ext ? ext.toUpperCase() : i18n.t('files:types.book');
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
    return i18n.t('files:types.ownerKinds.avatar');
  }
  if (file.owner === 'loose' || file.kind !== 'image') {
    return file.name || i18n.t('files:types.unnamed');
  }
  const images = sameRecord(files, file)
    .filter((item) => item.kind === 'image')
    .sort((left, right) => left.sort - right.sort || left.createdAt - right.createdAt);
  if (images.length <= 1) {
    return i18n.t('files:types.bodyImage');
  }
  const index = images.findIndex((item) => item.id === file.id) + 1;
  return i18n.t('files:types.bodyImageIndex', { index });
}

export type FileRoleKind = 'current' | 'encrypted' | 'older' | '';

export function fileRoleKind(file: UploadCatalog, files: UploadCatalog[]): FileRoleKind {
  if (file.kind !== 'book') {
    return '';
  }
  const current = readableBook(files, file);
  if (current?.id === file.id) {
    return 'current';
  }
  if (file.encrypted) {
    return 'encrypted';
  }
  return 'older';
}

export function fileRole(file: UploadCatalog, files: UploadCatalog[]) {
  const kind = fileRoleKind(file, files);
  return kind ? i18n.t(`files:role.${kind}`) : '';
}

export function readingNote(file: UploadCatalog, files: UploadCatalog[]) {
  if (file.owner !== 'book') {
    return '';
  }
  const books = bookFiles(sameRecord(files, file));
  if (books.length === 0) {
    return i18n.t('files:reading.none');
  }
  if (readableBook(files, file)) {
    return i18n.t('files:reading.readable');
  }
  if (books.every((item) => item.encrypted)) {
    return i18n.t('files:reading.encrypted');
  }
  return i18n.t('files:reading.noReadable');
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
    return displayName(left, files).localeCompare(displayName(right, files), currentLocale());
  }
  return right.createdAt - left.createdAt;
}

export function openLabel(file: UploadCatalog) {
  if (file.owner === 'book') {
    return i18n.t('files:open.book');
  }
  if (file.owner === 'book_note') {
    return i18n.t('files:open.bookNote');
  }
  if (file.owner === 'plan') {
    return i18n.t('files:open.plan');
  }
  if (file.owner === 'growth_entry') {
    return i18n.t('files:open.growth');
  }
  if (file.owner === 'member') {
    return i18n.t('files:open.member');
  }
  return i18n.t('files:open.journal');
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
    return i18n.t('files:host.bookNote');
  }
  if (file.owner === 'plan') {
    return i18n.t('files:host.plan');
  }
  if (file.owner === 'growth_entry') {
    return i18n.t('files:host.growth');
  }
  if (file.owner === 'member') {
    return i18n.t('files:host.member');
  }
  return i18n.t('files:host.journal');
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
      title: i18n.t('files:remove.looseTitle'),
      body: i18n.t('files:remove.looseBody')
    };
  }
  const title = file.ownerTitle.trim();
  const quoted = title ? i18n.t('files:remove.quote', { title }) : i18n.t('files:remove.thisRecord');
  if (file.ownerSealed) {
    return {
      blocked: true,
      title: i18n.t('files:remove.sealedTitle'),
      body: i18n.t('files:remove.sealedBody', { record: quoted })
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
        title: i18n.t('files:remove.bookCurrentTitle'),
        body: encryptedLeft
          ? i18n.t('files:remove.bookOnlyEncrypted', { quoted })
          : i18n.t('files:remove.bookNoReadable', { quoted })
      };
    }
    if (current && current.id !== file.id) {
      return {
        blocked: false,
        title: i18n.t('files:remove.bookTitle'),
        body: i18n.t('files:remove.bookStillUses', { quoted, format: bookFormatLabel(current.name) })
      };
    }
    return {
      blocked: false,
      title: i18n.t('files:remove.bookCurrentTitle'),
      body: i18n.t('files:remove.bookNoText', { quoted })
    };
  }
  if (file.owner === 'member') {
    const name = file.ownerTitle.trim();
    return {
      blocked: false,
      title: i18n.t('files:remove.avatarTitle'),
      body: name ? i18n.t('files:remove.avatarBody', { name }) : i18n.t('files:remove.avatarBodyNoName')
    };
  }
  if (file.kind === 'image') {
    return {
      blocked: false,
      title: i18n.t('files:remove.imageTitle'),
      body: title
        ? i18n.t('files:remove.fromHostNamed', { host: hostName(file), title })
        : i18n.t('files:remove.fromHost', { host: hostName(file) })
    };
  }
  return {
    blocked: false,
    title: i18n.t('files:remove.fileTitle'),
    body: title
      ? i18n.t('files:remove.fromHostNamed', { host: hostName(file), title })
      : i18n.t('files:remove.fromHost', { host: hostName(file) })
  };
}
