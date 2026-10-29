/** 把字节数写成 B / KB / MB / GB */
export function formatBytes(bytes: number) {
  if (!Number.isFinite(bytes) || bytes < 0) {
    return '未知';
  }
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  if (unit === 0) {
    return `${Math.round(value)} ${units[unit]}`;
  }
  return `${value.toFixed(1)} ${units[unit]}`;
}
