import { CaretDownIcon, CaretUpIcon, FilmStripIcon, TrashIcon } from '@phosphor-icons/react';
import type { ReactNode } from 'react';
import type { AudioTrack } from '@/bridge/task';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { Checkbox } from '@/components/checkbox';
import { Scroll } from '@/components/scroll';
import { TextField } from '@/components/text-field';
import { formatBytes } from '@/features/files/text';
import { splitPath } from '@/utils/path';
import type { VideoFile, VideoOp } from '../types';
import { nextClip } from '../types';
import { ClipFields } from './clip-fields';
import styles from './file-list.module.css';

type FileListProps = {
  files: VideoFile[];
  op: VideoOp;
  perFile: boolean;
  onChecked: (key: string, checked: boolean) => void;
  onChange: (file: VideoFile) => void;
  onMove: (index: number, delta: number) => void;
  onRemove: (key: string) => void;
  onSelectAll: () => void;
  onInvert: () => void;
  onClear: () => void;
  onDeleteSelected: () => void;
  start: ReactNode;
};

function formatDuration(seconds: number) {
  const total = Math.max(0, Math.round(seconds));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const remain = total % 60;
  const clock = `${String(minutes).padStart(2, '0')}:${String(remain).padStart(2, '0')}`;
  return hours > 0 ? `${hours}:${clock}` : clock;
}

function channelLabel(track: AudioTrack) {
  const layout = track.channelLayout.toLowerCase();
  if (layout.includes('stereo') || track.channels === 2) {
    return '立体声';
  }
  if (layout.includes('mono') || track.channels === 1) {
    return '单声道';
  }
  if (layout) {
    return track.channelLayout;
  }
  if (track.channels > 0) {
    return `${track.channels} 声道`;
  }
  return '';
}

function audioValue(file: VideoFile) {
  if (file.probeError) {
    return '—';
  }
  if (!file.info) {
    return '读取中';
  }
  const tracks = file.info.audioTracks;
  const first = tracks[0];
  if (!first) {
    return '无音轨';
  }
  const detail = [
    first.codec || '音频',
    channelLabel(first),
    tracks.length === 1 && first.sampleRate > 0 ? `${first.sampleRate} Hz` : ''
  ]
    .filter(Boolean)
    .join(' · ');
  return tracks.length === 1 ? detail : `${tracks.length} 条 · ${detail}`;
}

function textOf(file: VideoFile, value: string) {
  if (file.probeError) {
    return '—';
  }
  if (!file.info) {
    return '读取中';
  }
  return value || '—';
}

function Fact({ label, value }: { label: string; value: string }) {
  return (
    <span className={styles.fact}>
      <span className={styles.factLabel}>{label}</span>
      <span className={styles.factValue} title={value}>
        {value}
      </span>
    </span>
  );
}

export function FileList({
  files,
  op,
  perFile,
  onChecked,
  onChange,
  onMove,
  onRemove,
  onSelectAll,
  onInvert,
  onClear,
  onDeleteSelected,
  start
}: FileListProps) {
  const selected = files.filter((file) => file.checked);
  const baseKey = op === 'append' ? selected[0]?.key : undefined;
  const totalBytes = selected.reduce((sum, file) => sum + (file.size ?? 0), 0);
  const totalSeconds = selected.reduce((sum, file) => sum + (file.info?.duration ?? 0), 0);
  const empty = files.length === 0;

  return (
    <>
      <Scroll className={styles.sheet} axis="vertical" gutter>
        {empty ? <p className={styles.note}>还没有视频。选择文件或文件夹，也可以拖进窗口。</p> : null}
        {files.map((file, index) => {
          const name = splitPath(file.input).name;
          const info = file.info;
          const resolution = info && info.width > 0 && info.height > 0 ? `${info.width}×${info.height}` : '';
          const fps = info && info.fps > 0 ? `${Math.round(info.fps * 1000) / 1000} FPS` : '';
          const bitrate = info && info.bitrateKbps > 0 ? `${info.bitrateKbps} kbps` : '';
          return (
            <div className={styles.block} key={file.key}>
              <div className={styles.head}>
                <Checkbox
                  className={styles.check}
                  checked={file.checked}
                  aria-label={`选择 ${name}`}
                  onChange={(event) => onChecked(file.key, event.target.checked)}
                />
                <span className={styles.name} title={name}>
                  <FilmStripIcon weight="regular" />
                  <span className={styles.label}>{name}</span>
                  {file.key === baseKey ? <span className={styles.tag}>基准</span> : null}
                </span>
                <div className={styles.actions}>
                  {perFile ? null : (
                    <>
                      <IconButton label="上移" size="compact" disabled={index === 0} onClick={() => onMove(index, -1)}>
                        <CaretUpIcon weight="regular" />
                      </IconButton>
                      <IconButton
                        label="下移"
                        size="compact"
                        disabled={index === files.length - 1}
                        onClick={() => onMove(index, 1)}
                      >
                        <CaretDownIcon weight="regular" />
                      </IconButton>
                    </>
                  )}
                  <IconButton label="移除" tone="destructive" size="compact" onClick={() => onRemove(file.key)}>
                    <TrashIcon weight="regular" />
                  </IconButton>
                </div>
              </div>
              {file.probeError ? <p className={styles.probe}>{file.probeError}</p> : null}
              <div className={styles.facts}>
                <Fact label="大小" value={file.size == null ? '—' : formatBytes(file.size)} />
                <Fact label="时长" value={textOf(file, info ? formatDuration(info.duration) : '')} />
                <Fact label="分辨率" value={textOf(file, resolution)} />
                <Fact label="帧率" value={textOf(file, fps)} />
                <Fact label="视频编码" value={textOf(file, info?.videoCodec ?? '')} />
                <Fact label="像素格式" value={textOf(file, info?.pixFmt ?? '')} />
                <Fact label="码率" value={textOf(file, bitrate)} />
                <Fact label="音轨" value={audioValue(file)} />
              </div>
              {perFile ? (
                <TextField
                  label="输出路径"
                  value={file.output}
                  required
                  onChange={(event) => onChange({ ...file, output: event.target.value, edited: true })}
                />
              ) : null}
              {op === 'cut' ? (
                <div className={styles.clips}>
                  {file.segments.map((segment) => (
                    <ClipFields
                      key={segment.key}
                      start={segment.start}
                      end={segment.end}
                      canRemove={file.segments.length > 1}
                      onStart={(start) =>
                        onChange({
                          ...file,
                          segments: file.segments.map((part) => (part.key === segment.key ? { ...part, start } : part))
                        })
                      }
                      onEnd={(end) =>
                        onChange({
                          ...file,
                          segments: file.segments.map((part) => (part.key === segment.key ? { ...part, end } : part))
                        })
                      }
                      onRemove={() =>
                        onChange({
                          ...file,
                          segments: file.segments.filter((part) => part.key !== segment.key)
                        })
                      }
                    />
                  ))}
                  <Button
                    variant="text"
                    onClick={() => onChange({ ...file, segments: [...file.segments, nextClip(file.segments.at(-1))] })}
                  >
                    添加片段
                  </Button>
                </div>
              ) : null}
            </div>
          );
        })}
      </Scroll>
      <div className={styles.foot}>
        <div className={styles.footActions}>
          <Button variant="text" disabled={empty} onClick={onSelectAll}>
            全选
          </Button>
          <Button variant="text" disabled={empty} onClick={onInvert}>
            反选
          </Button>
          <Button variant="text" disabled={empty} onClick={onClear}>
            清空
          </Button>
          <Button variant="text" tone="destructive" disabled={selected.length === 0} onClick={onDeleteSelected}>
            删除
          </Button>
          <div className={styles.start}>{start}</div>
        </div>
        <p className={styles.total}>
          <span className={styles.statCount}>已选 {selected.length} 个</span>
          <span className={styles.statSize}>总大小 {formatBytes(totalBytes)}</span>
          <span className={styles.statTime}>总时长 {formatDuration(totalSeconds)}</span>
        </p>
      </div>
    </>
  );
}
