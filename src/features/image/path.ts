import { withSuffix } from '@/utils/path';

export function imageOutput(path: string, extension: '.avif' | '.jxl') {
  const next = withSuffix(path, '', extension);
  return next === path ? withSuffix(path, '_converted', extension) : next;
}
