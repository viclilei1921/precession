const imagePattern = /!\[[^\]]*\]\(<?upload:([A-Za-z0-9-]+)>?(?:\s+"[^"]*")?\)/g;
const videoPattern = /!\[[^\]]*\]\(<?video:([A-Za-z0-9-]+)>?(?:\s+"[^"]*")?\)/g;
const blockImage = /^(\s*)!\[([^\]]*)\]\((<[^>\n]+>|[^)\s]+)(?:\s+"((?:\\.|[^"\\])*)")?\)\s*$/;

function bareUrl(url: string) {
  const trimmed = url.trim();
  if (trimmed.startsWith('<') && trimmed.endsWith('>')) {
    return trimmed.slice(1, -1);
  }
  return trimmed;
}

/** `video:` 地址里的文件 id。不是视频引用时为空 */
export function videoIdFromUrl(url: string) {
  return /^video:([A-Za-z0-9-]+)$/.exec(bareUrl(url))?.[1] ?? '';
}

function isRatio(alt: string) {
  return /^\d+(?:\.\d+)?$/.test(alt);
}

function mapOutsideFences(source: string, map: (line: string) => string) {
  let fenced = false;
  return source
    .split('\n')
    .map((line) => {
      if (/^\s*(```|~~~)/.test(line)) {
        fenced = !fenced;
        return line;
      }
      return fenced ? line : map(line);
    })
    .join('\n');
}

/** 打开编辑器前，把整段图片的说明放进 title，避免被当成显示比例丢掉 */
export function markdownForEditor(source: string) {
  return mapOutsideFences(source, (line) => {
    const match = line.match(blockImage);
    if (!match) {
      return line;
    }
    const [, indent, alt, url, title] = match;
    if (videoIdFromUrl(url ?? '')) {
      return line;
    }
    if (title != null || isRatio(alt ?? '')) {
      return line;
    }
    const caption = (alt ?? '').replaceAll('\\', '\\\\').replaceAll('"', '\\"');
    return `${indent}![1.00](${url}${caption ? ` "${caption}"` : ''})`;
  });
}

/** 取出编辑器结果时，把 Crepe 的比例和 title 改回 `![说明](地址)` */
export function markdownFromEditor(source: string) {
  return mapOutsideFences(source, (line) => {
    const match = line.match(blockImage);
    if (!match) {
      return line;
    }
    const [, indent, alt, url, title] = match;
    if (videoIdFromUrl(url ?? '')) {
      return `${indent}![](${bareUrl(url ?? '')})`;
    }
    if (!isRatio(alt ?? '')) {
      return line;
    }
    const caption = (title ?? '').replaceAll('\\"', '"').replaceAll('\\\\', '\\');
    return `${indent}![${caption}](${url})`;
  });
}

/** 正文里第一张 `upload:` 图片。没有时为空 */
export function firstImageId(source: string) {
  return imageIds(source)[0] ?? '';
}

/** 正文里第一段 `video:` 视频。没有时为空 */
export function firstVideoId(source: string) {
  return videoIds(source)[0] ?? '';
}

/** 从正文里抽出 `video:` 视频 id，按出现顺序去重 */
export function videoIds(source: string): string[] {
  const ids: string[] = [];
  const visible = source.replace(/```[\s\S]*?```/g, ' ').replace(/~~~[\s\S]*?~~~/g, ' ');
  for (const match of visible.matchAll(videoPattern)) {
    const id = match[1];
    if (id && !ids.includes(id)) {
      ids.push(id);
    }
  }
  return ids;
}

/** 从正文里抽出 `upload:` 图片 id，按出现顺序去重 */
export function imageIds(source: string): string[] {
  const ids: string[] = [];
  const visible = source.replace(/```[\s\S]*?```/g, ' ').replace(/~~~[\s\S]*?~~~/g, ' ');
  for (const match of visible.matchAll(imagePattern)) {
    const id = match[1];
    if (id && !ids.includes(id)) {
      ids.push(id);
    }
  }
  return ids;
}

/** 收成一行可读文字。图片只留说明，没有说明就略过。标记、表格和分隔线都不留。 */
export function markdownPlain(source: string): string {
  const text = source
    .replace(/```[\s\S]*?```/g, ' ')
    .replace(/~~~[\s\S]*?~~~/g, ' ')
    .replace(
      /!\[([^\]]*)\]\((?:<[^>]+>|[^)\s]+)(?:\s+"((?:\\.|[^"\\])*)")?\)/g,
      (_match, alt: string, title?: string) => {
        const caption = title != null && isRatio(alt) ? title.replaceAll('\\"', '"') : alt;
        return caption;
      }
    )
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
    .replace(/\[([^\]]+)\]\[[^\]]*\]/g, '$1')
    .replace(/<https?:\/\/[^>\s]+>/g, ' ')
    .replace(/<\/?[A-Za-z][^>\n]*>/g, ' ')
    .replace(/`([^`]*)`/g, '$1')
    .replace(/\+\+([\s\S]+?)\+\+/g, '$1')
    .replace(/==([\s\S]+?)==/g, '$1')
    .replace(/~~([\s\S]+?)~~/g, '$1')
    .replace(/^#{1,6}\s+/gm, '')
    .replace(/^\s*[-*+]\s+\[[ xX]\]\s+/gm, '')
    .replace(/^\s*[-*+]\s+/gm, '')
    .replace(/^\s*\d+\.\s+/gm, '')
    .replace(/^\s*>\s?/gm, '')
    .replace(/^\s*([-*_])\1{2,}\s*$/gm, ' ')
    .replace(/^\s*\|?[\s|:-]+\|[\s|:-]*$/gm, ' ')
    .replace(/\|/g, ' ')
    .replace(/\[\^[^\]]+\]/g, ' ')
    .replace(/!\[[^\]]*\]/g, ' ')
    .replace(/[*_~>#\\[\]]/g, '');
  return text
    .replace(/\s+/g, ' ')
    .replace(/\s+([，。！？、；：,.!?;:])/g, '$1')
    .trim();
}
