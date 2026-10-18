/** 把路径拆成目录和主文件名。 */
export function splitPath(path: string) {
  const slash = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
  const name = path.slice(slash + 1);
  const dot = name.lastIndexOf('.');
  return {
    dir: path.slice(0, slash + 1),
    stem: dot > 0 ? name.slice(0, dot) : name,
    name
  };
}

/** 在原文件名后加后缀，并换成指定扩展名。 */
export function withSuffix(path: string, suffix: string, extension: string) {
  const { dir, stem } = splitPath(path);
  return `${dir}${stem}${suffix}${extension}`;
}
