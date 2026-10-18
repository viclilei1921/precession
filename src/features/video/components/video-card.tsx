import {
  CaretDownIcon,
  CaretUpIcon,
  ClockIcon,
  FileIcon,
  FilmStripIcon,
  FrameCornersIcon,
  GaugeIcon,
  GridFourIcon,
  SpeakerHighIcon,
  TrashIcon,
  WaveformIcon
} from '@phosphor-icons/react';
import type { ReactNode } from 'react';
import { Button } from '@/components/button';
import { Card } from '@/components/card';
import { TextField } from '@/components/text-field';
import { splitPath } from '@/utils/path';
import type { VideoFile, VideoOp } from '../types';
import { nextClip } from '../types';
import { ClipFields } from './clip-fields';
import styles from './video-card.module.css';

type VideoCardProps = {
  file: VideoFile;
  op: VideoOp;
  index: number;
  total: number;
  perFile: boolean;
  onChange: (file: VideoFile) => void;
  onMove: (delta: number) => void;
  onRemove: () => void;
};

function formatDuration(seconds: number) {
  if (!Number.isFinite(seconds) || seconds <= 0) {
    return '时长未知';
  }
  const total = Math.round(seconds);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const remain = total % 60;
  const clock = `${String(minutes).padStart(2, '0')}:${String(remain).padStart(2, '0')}`;
  return hours > 0 ? `${hours}:${clock}` : clock;
}

function audioValue(file: VideoFile) {
  const tracks = file.info?.audioTracks ?? [];
  if (tracks.length === 0) {
    return '无音轨';
  }
  const first = tracks[0];
  if (!first) {
    return '无音轨';
  }
  if (tracks.length === 1) {
    const rate = first.sampleRate > 0 ? ` · ${first.sampleRate} Hz` : '';
    return `${first.codec || '音频'}${rate}`;
  }
  return `${tracks.length} 条 · ${first.codec || '音频'}`;
}

function InfoCell({ icon, label, value }: { icon: ReactNode; label: string; value: string }) {
  return (
    <div className={styles.cell}>
      <span className={styles.label}>
        {icon}
        {label}
      </span>
      <span className={styles.value} title={value}>
        {value}
      </span>
    </div>
  );
}

/** 一个输入视频：探测信息、输出路径，裁剪时带片段。 */
export function VideoCard({ file, op, index, total, perFile, onChange, onMove, onRemove }: VideoCardProps) {
  const info = file.info;
  const name = splitPath(file.input).name;
  const extra = op === 'append' && index === 0 ? '基准' : undefined;

  return (
    <Card title={name} extra={extra}>
      {file.probeError ? <p className={styles.error}>{file.probeError}</p> : null}
      <div className={styles.grid}>
        <InfoCell icon={<FileIcon weight="regular" />} label="文件" value={name} />
        <InfoCell
          icon={<ClockIcon weight="regular" />}
          label="时长"
          value={info ? formatDuration(info.duration) : '读取中'}
        />
        <InfoCell
          icon={<FrameCornersIcon weight="regular" />}
          label="分辨率"
          value={info ? `${info.width}×${info.height}` : '读取中'}
        />
        <InfoCell
          icon={<GaugeIcon weight="regular" />}
          label="帧率"
          value={info && info.fps > 0 ? `${info.fps} FPS` : info ? '—' : '读取中'}
        />
      </div>
      <div className={styles.rule} />
      <div className={styles.grid}>
        <InfoCell
          icon={<FilmStripIcon weight="regular" />}
          label="视频编码"
          value={info?.videoCodec || (info ? '—' : '读取中')}
        />
        <InfoCell
          icon={<GridFourIcon weight="regular" />}
          label="像素格式"
          value={info?.pixFmt || (info ? '—' : '读取中')}
        />
        <InfoCell
          icon={<WaveformIcon weight="regular" />}
          label="码率"
          value={info && info.bitrateKbps > 0 ? `${info.bitrateKbps} kbps` : info ? '—' : '读取中'}
        />
        <InfoCell icon={<SpeakerHighIcon weight="regular" />} label="音轨" value={info ? audioValue(file) : '读取中'} />
      </div>
      {perFile ? (
        <TextField
          label="输出路径"
          value={file.output}
          required
          onChange={(event) => onChange({ ...file, output: event.target.value, edited: true })}
        />
      ) : null}
      {op === 'cut'
        ? file.segments.map((segment, segmentIndex) => (
            <ClipFields
              key={segment.key}
              index={segmentIndex}
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
          ))
        : null}
      <div className={styles.actions}>
        {op === 'cut' ? (
          <Button
            variant="text"
            onClick={() => onChange({ ...file, segments: [...file.segments, nextClip(file.segments.at(-1))] })}
          >
            添加片段
          </Button>
        ) : null}
        {perFile ? null : (
          <>
            <Button variant="text" disabled={index === 0} onClick={() => onMove(-1)}>
              <CaretUpIcon weight="regular" />
              上移
            </Button>
            <Button variant="text" disabled={index === total - 1} onClick={() => onMove(1)}>
              <CaretDownIcon weight="regular" />
              下移
            </Button>
          </>
        )}
        <Button variant="text" tone="destructive" className={styles.remove} onClick={onRemove}>
          <TrashIcon weight="regular" />
          移除
        </Button>
      </div>
    </Card>
  );
}
