/** 把路径拆成目录、主文件名和扩展名。 */
export function splitPath(path: string) {
  const slash = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
  const name = path.slice(slash + 1);
  const dot = name.lastIndexOf('.');
  return {
    dir: path.slice(0, slash + 1),
    stem: dot > 0 ? name.slice(0, dot) : name
  };
}

/** 在原文件名后加后缀，并换成指定扩展名。 */
export function withSuffix(path: string, suffix: string, extension: string) {
  const { dir, stem } = splitPath(path);
  return `${dir}${stem}${suffix}${extension}`;
}

export function containerExtension(container: 'mp4' | 'mov' | 'mpegTs') {
  if (container === 'mov') {
    return '.mov';
  }
  if (container === 'mpegTs') {
    return '.ts';
  }
  return '.mp4';
}

export function encryptOutput(path: string) {
  return path.toLowerCase().endsWith('.enc') ? path : `${path}.enc`;
}

export function decryptOutput(path: string) {
  return path.toLowerCase().endsWith('.enc') ? path.slice(0, -4) : `${path}.dec`;
}

export function imageOutput(path: string, extension: '.avif' | '.jxl') {
  const next = withSuffix(path, '', extension);
  return next === path ? withSuffix(path, '_converted', extension) : next;
}
