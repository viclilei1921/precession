import type { VideoInfo } from '@/bridge/task';

export type VideoOp = 'convert' | 'cut' | 'merge' | 'append';

export type Clip = {
  key: string;
  start: number;
  end: number;
};

export type VideoFile = {
  key: string;
  input: string;
  output: string;
  edited: boolean;
  info: VideoInfo | null;
  probeError: string | null;
  segments: Clip[];
};

let fileKey = 0;

export function nextFileKey() {
  fileKey += 1;
  return `video-${fileKey}`;
}

export function nextClip(previous?: Clip): Clip {
  fileKey += 1;
  const start = previous ? previous.end + 5 : 0;
  return { key: `clip-${fileKey}`, start, end: start + 5 };
}
