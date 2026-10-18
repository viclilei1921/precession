import { withSuffix } from '@/utils/path';

export function containerExtension(container: 'mp4' | 'mov' | 'mpegTs') {
  if (container === 'mov') {
    return '.mov';
  }
  if (container === 'mpegTs') {
    return '.ts';
  }
  return '.mp4';
}

export function outputExtension(mode: 'default' | 'custom', container: 'mp4' | 'mov' | 'mpegTs', codec: string) {
  if (mode !== 'custom') {
    return '.mp4';
  }
  return containerExtension(codec === 'vp9' && container === 'mpegTs' ? 'mp4' : container);
}

export function videoOutput(path: string, suffix: string, extension: string) {
  return withSuffix(path, suffix, extension);
}
