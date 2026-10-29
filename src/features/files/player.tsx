import { FileIcon, LockIcon, PlayIcon } from '@phosphor-icons/react';
import type { CSSProperties } from 'react';
import { useLayoutEffect, useRef, useState } from 'react';
import { flushSync } from 'react-dom';
import type { UploadCatalog } from '@/bridge/upload';
import { IconButton } from '@/components/button/icon';
import { UnlockForm, useMediaAccess } from './access';
import styles from './player.module.css';
import { bufferedEnd, cueTime, readPreviewTime, writePreviewTime } from './preview';
import { Scrub } from './scrub';
import { bookFormatLabel } from './text';

/**
 * 右侧小窗。
 * 图片和视频只留中间的预览按钮。视频不能播放，只能拖进度条看画面。
 */
export function Player({
  file,
  sync = 0,
  onPreview
}: {
  file: UploadCatalog;
  /** 大预览关掉后加一，小窗跟到刚看到的那一帧 */
  sync?: number;
  onPreview: () => void;
}) {
  const access = useMediaAccess(file);
  const videoRef = useRef<HTMLVideoElement>(null);
  const [ratio, setRatio] = useState<number | null>(null);
  const [current, setCurrent] = useState(0);
  const [duration, setDuration] = useState(0);
  const [buffered, setBuffered] = useState(0);
  const [playbackError, setPlaybackError] = useState('');
  const { playable, phase, message, src, pending } = access;
  const frame = ratio && ratio > 0 ? ({ '--frame': ratio } as CSSProperties) : undefined;
  const ready = playable && phase === 'ready' && src && !playbackError;

  // biome-ignore lint/correctness/useExhaustiveDependencies: sync 只用来触发重新对帧，函数体读的是记下的进度
  useLayoutEffect(() => {
    const node = videoRef.current;
    if (!node || !src) {
      return;
    }
    const place = () => {
      if (node.videoWidth > 0 && node.videoHeight > 0) {
        setRatio(node.videoWidth / node.videoHeight);
      }
      const time = cueTime(node.duration, readPreviewTime(file.id));
      if (Number.isFinite(node.duration) && Math.abs(node.currentTime - time) > 0.05) {
        node.currentTime = time;
      }
      node.pause();
      writePreviewTime(file.id, time);
      setDuration(Number.isFinite(node.duration) ? node.duration : 0);
      setCurrent(time);
    };
    if (node.readyState >= 1) {
      place();
    }
    node.addEventListener('loadedmetadata', place);
    return () => node.removeEventListener('loadedmetadata', place);
  }, [file.id, src, sync]);

  function seek(value: number) {
    const node = videoRef.current;
    if (!node) {
      return;
    }
    const limit = Number.isFinite(node.duration) ? node.duration : value;
    const time = Math.min(Math.max(0, value), limit);
    node.currentTime = time;
    node.pause();
    writePreviewTime(file.id, time);
    setCurrent(time);
  }

  function rememberImage(node: HTMLImageElement) {
    if (node.naturalWidth > 0 && node.naturalHeight > 0) {
      setRatio(node.naturalWidth / node.naturalHeight);
    }
  }

  return (
    <div className={styles.stage} style={frame}>
      <div className={styles.view}>
        {file.ownerSealed ? (
          <span className={styles.plate}>
            <LockIcon className={styles.glyph} weight="regular" />
            所属记录已封存
          </span>
        ) : null}
        {!file.ownerSealed && !playable && file.kind === 'book' ? (
          <span className={styles.plate}>
            <strong className={styles.bookMark}>{bookFormatLabel(file.name)}</strong>
          </span>
        ) : null}
        {!file.ownerSealed && !playable && file.kind !== 'book' ? (
          <span className={styles.plate}>
            <FileIcon className={styles.glyph} weight="regular" />
            文件
          </span>
        ) : null}
        {playable && (phase === 'check' || (phase === 'ready' && !src && pending)) ? (
          <span className={styles.plate}>正在打开…</span>
        ) : null}
        {playable && phase === 'error' ? <span className={styles.plate}>{message || '打不开'}</span> : null}
        {playable && phase === 'need' ? (
          <div className={styles.gate}>
            <UnlockForm access={access} />
          </div>
        ) : null}
        {playable && phase === 'ready' && !src && !pending ? (
          <span className={styles.plate}>预览暂时不可用</span>
        ) : null}
        {playbackError ? <span className={styles.plate}>{playbackError}</span> : null}
        {ready && file.kind === 'image' ? (
          <img
            className={styles.media}
            src={src}
            alt=""
            onLoad={(event) => rememberImage(event.currentTarget)}
            ref={(node) => {
              if (node?.complete) {
                rememberImage(node);
              }
            }}
            onError={() => setPlaybackError('这张图片打不开')}
          />
        ) : null}
        {ready && file.kind === 'video' ? (
          // biome-ignore lint/a11y/useMediaCaption: 用户自己的视频没有字幕轨
          <video
            ref={videoRef}
            className={styles.media}
            src={src}
            playsInline
            preload="auto"
            inert
            onPlay={(event) => event.currentTarget.pause()}
            onSeeked={(event) => setCurrent(event.currentTarget.currentTime)}
            onProgress={(event) => setBuffered(bufferedEnd(event.currentTarget))}
            onError={() => setPlaybackError('这个视频打不开')}
          />
        ) : null}
        {ready ? (
          <IconButton
            className={styles.peek}
            label="预览"
            onClick={() => {
              flushSync(onPreview);
            }}
          >
            <PlayIcon weight="fill" />
          </IconButton>
        ) : null}
        {ready && file.kind === 'video' ? (
          <div className={styles.miniBar}>
            <Scrub tip current={current} duration={duration} buffered={buffered} onSeek={seek} />
          </div>
        ) : null}
      </div>
    </div>
  );
}
