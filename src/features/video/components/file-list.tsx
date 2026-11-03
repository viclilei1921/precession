import { CaretDownIcon, CaretUpIcon, FilmStripIcon, TrashIcon } from '@phosphor-icons/react';
import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import type { AudioTrack } from '@/bridge/task';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { Checkbox } from '@/components/checkbox';
import { Scroll } from '@/components/scroll';
import { TextField } from '@/components/text-field';
import { formatBytes } from '@/features/files/text';
import i18n from '@/i18n';
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
    return i18n.t('video:fileList.stereo');
  }
  if (layout.includes('mono') || track.channels === 1) {
    return i18n.t('video:fileList.mono');
  }
  if (layout) {
    return track.channelLayout;
  }
  if (track.channels > 0) {
    return i18n.t('video:fileList.channels', { count: track.channels });
  }
  return '';
}

function audioValue(file: VideoFile) {
  if (file.probeError) {
    return '—';
  }
  if (!file.info) {
    return i18n.t('video:fileList.reading');
  }
  const tracks = file.info.audioTracks;
  const first = tracks[0];
  if (!first) {
    return i18n.t('video:fileList.noAudioTrack');
  }
  const detail = [
    first.codec || i18n.t('video:fileList.audioFallback'),
    channelLabel(first),
    tracks.length === 1 && first.sampleRate > 0 ? `${first.sampleRate} Hz` : ''
  ]
    .filter(Boolean)
    .join(' · ');
  return tracks.length === 1 ? detail : i18n.t('video:fileList.audioTracks', { count: tracks.length, detail });
}

function textOf(file: VideoFile, value: string) {
  if (file.probeError) {
    return '—';
  }
  if (!file.info) {
    return i18n.t('video:fileList.reading');
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
  const { t } = useTranslation('video');
  const selected = files.filter((file) => file.checked);
  const baseKey = op === 'append' ? selected[0]?.key : undefined;
  const totalBytes = selected.reduce((sum, file) => sum + (file.size ?? 0), 0);
  const totalSeconds = selected.reduce((sum, file) => sum + (file.info?.duration ?? 0), 0);
  const empty = files.length === 0;

  return (
    <>
      <Scroll className={styles.sheet} axis="vertical" gutter>
        {empty ? <p className={styles.note}>{t('fileList.empty')}</p> : null}
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
                  aria-label={t('fileList.select', { name })}
                  onChange={(event) => onChecked(file.key, event.target.checked)}
                />
                <span className={styles.name} title={name}>
                  <FilmStripIcon weight="regular" />
                  <span className={styles.label}>{name}</span>
                  {file.key === baseKey ? <span className={styles.tag}>{t('fileList.base')}</span> : null}
                </span>
                <div className={styles.actions}>
                  {perFile ? null : (
                    <>
                      <IconButton
                        label={t('fileList.moveUp')}
                        size="compact"
                        disabled={index === 0}
                        onClick={() => onMove(index, -1)}
                      >
                        <CaretUpIcon weight="regular" />
                      </IconButton>
                      <IconButton
                        label={t('fileList.moveDown')}
                        size="compact"
                        disabled={index === files.length - 1}
                        onClick={() => onMove(index, 1)}
                      >
                        <CaretDownIcon weight="regular" />
                      </IconButton>
                    </>
                  )}
                  <IconButton
                    label={t('fileList.remove')}
                    tone="destructive"
                    size="compact"
                    onClick={() => onRemove(file.key)}
                  >
                    <TrashIcon weight="regular" />
                  </IconButton>
                </div>
              </div>
              {file.probeError ? <p className={styles.probe}>{file.probeError}</p> : null}
              <div className={styles.facts}>
                <Fact label={t('fileList.size')} value={file.size == null ? '—' : formatBytes(file.size)} />
                <Fact label={t('fileList.duration')} value={textOf(file, info ? formatDuration(info.duration) : '')} />
                <Fact label={t('fileList.resolution')} value={textOf(file, resolution)} />
                <Fact label={t('fileList.frameRate')} value={textOf(file, fps)} />
                <Fact label={t('fileList.videoCodec')} value={textOf(file, info?.videoCodec ?? '')} />
                <Fact label={t('fileList.pixelFormat')} value={textOf(file, info?.pixFmt ?? '')} />
                <Fact label={t('fileList.bitrate')} value={textOf(file, bitrate)} />
                <Fact label={t('fileList.audioTrack')} value={audioValue(file)} />
              </div>
              {perFile ? (
                <TextField
                  label={t('page.outputPath')}
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
                    {t('fileList.addClip')}
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
            {t('fileList.selectAll')}
          </Button>
          <Button variant="text" disabled={empty} onClick={onInvert}>
            {t('fileList.invert')}
          </Button>
          <Button variant="text" disabled={empty} onClick={onClear}>
            {t('fileList.clear')}
          </Button>
          <Button variant="text" tone="destructive" disabled={selected.length === 0} onClick={onDeleteSelected}>
            {t('fileList.delete')}
          </Button>
          <div className={styles.start}>{start}</div>
        </div>
        <p className={styles.total}>
          <span className={styles.statCount}>{t('fileList.selected', { count: selected.length })}</span>
          <span className={styles.statSize}>{t('fileList.totalSize', { size: formatBytes(totalBytes) })}</span>
          <span className={styles.statTime}>
            {t('fileList.totalDuration', { duration: formatDuration(totalSeconds) })}
          </span>
        </p>
      </div>
    </>
  );
}
