import {
  CaretLeftIcon,
  CaretRightIcon,
  LockIcon,
  MinusIcon,
  PauseIcon,
  PlayIcon,
  PlusIcon,
  SpeakerHighIcon,
  SpeakerSlashIcon,
  XIcon
} from '@phosphor-icons/react';
import clsx from 'clsx';
import type { PointerEvent as ReactPointerEvent } from 'react';
import { useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import type { UploadCatalog } from '@/bridge/upload';
import { IconButton } from '@/components/button/icon';
import { ScrimButton } from '@/components/button/scrim';
import { UnlockForm, useMediaAccess } from './access';
import { bufferedEnd, clock, cueTime, readPreviewTime, writePreviewTime } from './preview';
import { Scrub } from './scrub';
import { displayName, formatBytes, moduleLabel, moduleOf } from './text';
import styles from './viewer.module.css';

type MediaViewerProps = {
  file: UploadCatalog;
  /** 全部文件，用来算图片在记录里的名字 */
  catalog: UploadCatalog[];
  /** 当前列表。左右切换只在其中的图片和视频里走 */
  files: UploadCatalog[];
  autoPlay: boolean;
  onSelect: (id: string) => void;
  onClose: () => void;
};

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

function neighbors(file: UploadCatalog, files: UploadCatalog[]) {
  const list = files.filter((item) => item.kind === 'image' || item.kind === 'video');
  const index = list.findIndex((item) => item.id === file.id);
  return {
    index,
    total: list.length,
    prev: index > 0 ? list[index - 1] : undefined,
    next: index >= 0 && index < list.length - 1 ? list[index + 1] : undefined
  };
}

/** 盖住标题栏以下的图片 / 视频预览。 */
export function MediaViewer({ file, catalog, files, autoPlay, onSelect, onClose }: MediaViewerProps) {
  const titleId = useId();
  const rootRef = useRef<HTMLDivElement>(null);
  const [shown, setShown] = useState(false);
  const [volume, setVolume] = useState(1);
  const [muted, setMuted] = useState(false);

  useEffect(() => {
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    rootRef.current?.focus({ preventScroll: true });
    const frame = requestAnimationFrame(() => setShown(true));
    return () => {
      cancelAnimationFrame(frame);
      previous?.focus();
    };
  }, []);

  return createPortal(
    <div
      ref={rootRef}
      className={styles.viewer}
      data-open={shown ? 'true' : undefined}
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
      tabIndex={-1}
    >
      <ViewerFrame
        key={file.id}
        file={file}
        catalog={catalog}
        files={files}
        autoPlay={autoPlay}
        titleId={titleId}
        volume={volume}
        muted={muted}
        onVolume={setVolume}
        onMuted={setMuted}
        onSelect={onSelect}
        onClose={onClose}
      />
    </div>,
    document.body
  );
}

function ViewerFrame({
  file,
  catalog,
  files,
  autoPlay,
  titleId,
  volume,
  muted,
  onVolume,
  onMuted,
  onSelect,
  onClose
}: MediaViewerProps & {
  titleId: string;
  volume: number;
  muted: boolean;
  onVolume: (value: number) => void;
  onMuted: (value: boolean) => void;
}) {
  const access = useMediaAccess(file);
  const stageRef = useRef<HTMLDivElement>(null);
  const videoRef = useRef<HTMLVideoElement>(null);
  const drag = useRef<{ x: number; y: number; panX: number; panY: number } | null>(null);
  const view = useRef({ shown: 0, x: 0, y: 0 });
  const [box, setBox] = useState({ w: 0, h: 0 });
  const [natural, setNatural] = useState<{ w: number; h: number } | null>(null);
  const [videoSize, setVideoSize] = useState<{ w: number; h: number } | null>(null);
  const [mode, setMode] = useState<'fit' | 'free'>('fit');
  const [zoom, setZoom] = useState(1);
  const [panX, setPanX] = useState(0);
  const [panY, setPanY] = useState(0);
  const [paused, setPaused] = useState(true);
  const [armed, setArmed] = useState(autoPlay);
  const [current, setCurrent] = useState(0);
  const [duration, setDuration] = useState(0);
  const [buffered, setBuffered] = useState(0);
  const [playbackError, setPlaybackError] = useState('');
  const { playable, phase, message, src, pending } = access;
  const spot = neighbors(file, files);
  const name = displayName(file, catalog);
  const fit = natural && box.w > 1 && box.h > 1 ? Math.min(box.w / natural.w, box.h / natural.h) : 0;
  const shown = mode === 'fit' ? fit : zoom;
  const width = natural ? natural.w * shown : 0;
  const height = natural ? natural.h * shown : 0;
  const x = clamp(panX, -Math.max(0, (width - box.w) / 2), Math.max(0, (width - box.w) / 2));
  const y = clamp(panY, -Math.max(0, (height - box.h) / 2), Math.max(0, (height - box.h) / 2));
  const canPan = width > box.w + 1 || height > box.h + 1;
  const videoFit = videoSize && box.w > 1 && box.h > 1 ? Math.min(box.w / videoSize.w, box.h / videoSize.h) : 0;
  view.current = { shown, x, y };
  const ready = playable && phase === 'ready' && src && !playbackError;
  const silent = muted || volume === 0;
  const meta =
    file.kind === 'image'
      ? [natural ? `${natural.w} × ${natural.h}` : '', formatBytes(file.size), moduleLabel(moduleOf(file.owner))]
          .filter(Boolean)
          .join(' · ')
      : [duration > 0 ? clock(duration) : '', formatBytes(file.size), moduleLabel(moduleOf(file.owner))]
          .filter(Boolean)
          .join(' · ');

  function zoomBy(factor: number) {
    const prev = shown || fit;
    if (!prev) {
      return;
    }
    setMode('free');
    setZoom(clamp(prev * factor, 0.05, 6));
  }

  function toggleFit() {
    if (mode === 'fit') {
      setMode('free');
      setZoom(1);
      return;
    }
    setMode('fit');
    setPanX(0);
    setPanY(0);
  }

  function seek(time: number) {
    const node = videoRef.current;
    if (!node || !Number.isFinite(node.duration)) {
      return;
    }
    const next = clamp(time, 0, node.duration);
    node.currentTime = next;
    writePreviewTime(file.id, next);
    setCurrent(next);
  }

  function toggle() {
    const node = videoRef.current;
    if (!node) {
      return;
    }
    if (node.ended) {
      node.currentTime = 0;
    }
    if (node.paused) {
      void node.play().catch(() => setPaused(true));
      return;
    }
    node.pause();
  }

  function step(delta: number) {
    const next = delta < 0 ? spot.prev : spot.next;
    if (next) {
      onSelect(next.id);
    }
  }

  const act = useRef({
    close: onClose,
    kind: file.kind,
    ready,
    toggle,
    seek,
    step,
    zoomBy
  });
  act.current = { close: onClose, kind: file.kind, ready, toggle, seek, step, zoomBy };

  useLayoutEffect(() => {
    const node = stageRef.current;
    if (!node) {
      return;
    }
    const watch = () => {
      const w = node.clientWidth;
      const h = node.clientHeight;
      setBox((currentBox) => (currentBox.w === w && currentBox.h === h ? currentBox : { w, h }));
    };
    watch();
    const observer = new ResizeObserver(watch);
    observer.observe(node);
    return () => observer.disconnect();
  }, []);

  useLayoutEffect(() => {
    const node = videoRef.current;
    if (!node || file.kind !== 'video' || !src) {
      return;
    }
    let cancel = false;
    const place = () => {
      if (node.videoWidth > 0 && node.videoHeight > 0) {
        setVideoSize({ w: node.videoWidth, h: node.videoHeight });
      }
      const time = cueTime(node.duration, readPreviewTime(file.id));
      if (Number.isFinite(node.duration) && Math.abs(node.currentTime - time) > 0.05) {
        node.currentTime = time;
      }
      if (!autoPlay) {
        node.pause();
      }
      writePreviewTime(file.id, time);
      setDuration(Number.isFinite(node.duration) ? node.duration : 0);
      setCurrent(time);
    };
    if (node.readyState >= 1) {
      place();
    } else {
      node.addEventListener('loadedmetadata', place);
    }
    if (autoPlay) {
      const pending = node.play();
      if (!pending) {
        setArmed(false);
      } else {
        void pending.then(
          () => {
            if (!cancel) {
              setArmed(false);
            }
          },
          () => {
            if (!cancel) {
              setArmed(false);
            }
          }
        );
      }
    }
    return () => {
      cancel = true;
      node.removeEventListener('loadedmetadata', place);
      if (node.readyState >= 1) {
        writePreviewTime(file.id, node.currentTime);
      }
      node.pause();
    };
  }, [autoPlay, file.id, file.kind, src]);

  useEffect(() => {
    const node = videoRef.current;
    if (!node || !src) {
      return;
    }
    node.volume = volume;
    node.muted = muted || volume === 0;
  }, [muted, src, volume]);

  useEffect(() => {
    const node = stageRef.current;
    if (!node || file.kind !== 'image') {
      return;
    }
    const onWheel = (event: WheelEvent) => {
      if (event.ctrlKey) {
        return;
      }
      const target = event.target;
      if (target instanceof Element && target.closest(`.${styles.dock}`)) {
        return;
      }
      event.preventDefault();
      const currentView = view.current;
      if (!currentView.shown) {
        return;
      }
      const next = clamp(currentView.shown * (event.deltaY < 0 ? 1.1 : 1 / 1.1), 0.05, 6);
      const rect = node.getBoundingClientRect();
      const cx = event.clientX - rect.left - rect.width / 2;
      const cy = event.clientY - rect.top - rect.height / 2;
      const nextX = cx - ((cx - currentView.x) * next) / currentView.shown;
      const nextY = cy - ((cy - currentView.y) * next) / currentView.shown;
      view.current = { shown: next, x: nextX, y: nextY };
      setPanX(nextX);
      setPanY(nextY);
      setMode('free');
      setZoom(next);
    };
    node.addEventListener('wheel', onWheel, { passive: false });
    return () => node.removeEventListener('wheel', onWheel);
  }, [file.kind]);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        event.preventDefault();
        event.stopPropagation();
        act.current.close();
        return;
      }
      const target = event.target;
      if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) {
        return;
      }
      if (event.key === ' ' && target instanceof HTMLButtonElement) {
        return;
      }
      const api = act.current;
      if (!api.ready) {
        return;
      }
      if (api.kind === 'video') {
        if (event.key === ' ') {
          event.preventDefault();
          api.toggle();
        } else if (event.key === 'ArrowLeft' && event.shiftKey) {
          event.preventDefault();
          api.step(-1);
        } else if (event.key === 'ArrowRight' && event.shiftKey) {
          event.preventDefault();
          api.step(1);
        } else if (event.key === 'ArrowLeft') {
          event.preventDefault();
          api.seek((videoRef.current?.currentTime ?? 0) - 5);
        } else if (event.key === 'ArrowRight') {
          event.preventDefault();
          api.seek((videoRef.current?.currentTime ?? 0) + 5);
        } else if (event.key === 'Home') {
          event.preventDefault();
          api.seek(0);
        } else if (event.key === 'End') {
          event.preventDefault();
          api.seek(videoRef.current?.duration ?? 0);
        }
      }
      if (api.kind === 'image') {
        if (event.key === 'ArrowLeft') {
          event.preventDefault();
          api.step(-1);
        } else if (event.key === 'ArrowRight') {
          event.preventDefault();
          api.step(1);
        } else if (event.key === '+' || event.key === '=') {
          event.preventDefault();
          api.zoomBy(1.25);
        } else if (event.key === '-' || event.key === '_') {
          event.preventDefault();
          api.zoomBy(1 / 1.25);
        }
      }
    }
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, []);

  function rememberImage(node: HTMLImageElement) {
    if (node.naturalWidth <= 0 || node.naturalHeight <= 0) {
      return;
    }
    setNatural((currentSize) =>
      currentSize?.w === node.naturalWidth && currentSize.h === node.naturalHeight
        ? currentSize
        : { w: node.naturalWidth, h: node.naturalHeight }
    );
  }

  function onPanStart(event: ReactPointerEvent<HTMLImageElement>) {
    if (!canPan || event.button !== 0) {
      return;
    }
    drag.current = { x: event.clientX, y: event.clientY, panX: x, panY: y };
    event.currentTarget.setPointerCapture(event.pointerId);
  }

  function onPanMove(event: ReactPointerEvent<HTMLImageElement>) {
    const origin = drag.current;
    if (!origin) {
      return;
    }
    setPanX(origin.panX + event.clientX - origin.x);
    setPanY(origin.panY + event.clientY - origin.y);
  }

  return (
    <>
      <header className={styles.head}>
        <span className={styles.kind}>{file.kind === 'video' ? '视频' : '图片'}</span>
        <div className={styles.name}>
          <strong id={titleId} className={styles.title}>
            {name}
          </strong>
          <span className={styles.sub}>{meta}</span>
        </div>
        {spot.index >= 0 ? (
          <span className={styles.index}>
            {spot.index + 1} / {spot.total}
          </span>
        ) : null}
        <div className={styles.tools}>
          {ready && file.kind === 'image' ? (
            <div className={styles.zoom}>
              <IconButton className={styles.tool} label="缩小" size="compact" onClick={() => zoomBy(1 / 1.25)}>
                <MinusIcon weight="regular" />
              </IconButton>
              <button type="button" className={styles.zoomLabel} title="在适应和原大之间切换" onClick={toggleFit}>
                {mode === 'fit' ? '适应' : `${Math.round(shown * 100)}%`}
              </button>
              <IconButton className={styles.tool} label="放大" size="compact" onClick={() => zoomBy(1.25)}>
                <PlusIcon weight="regular" />
              </IconButton>
            </div>
          ) : null}
          <IconButton className={styles.tool} label="关闭" onClick={onClose}>
            <XIcon weight="regular" />
          </IconButton>
        </div>
      </header>
      <div ref={stageRef} className={styles.stage}>
        <ScrimButton onClick={onClose} />
        {file.ownerSealed ? (
          <span className={styles.note}>
            <LockIcon className={styles.glyph} weight="regular" />
            所属记录已封存
          </span>
        ) : null}
        {playable && (phase === 'check' || (phase === 'ready' && !src && pending)) ? (
          <span className={styles.note}>正在打开…</span>
        ) : null}
        {playable && phase === 'error' ? <span className={styles.note}>{message || '打不开'}</span> : null}
        {playable && phase === 'need' ? (
          <div className={styles.gate}>
            <UnlockForm access={access} />
          </div>
        ) : null}
        {playable && phase === 'ready' && !src && !pending ? <span className={styles.note}>预览暂时不可用</span> : null}
        {playbackError ? <span className={styles.note}>{playbackError}</span> : null}
        {ready && file.kind === 'image' ? (
          <img
            className={styles.shot}
            data-ready={shown > 0 ? 'true' : undefined}
            data-pan={canPan ? 'true' : undefined}
            src={src}
            alt=""
            aria-hidden="true"
            draggable={false}
            style={
              shown > 0
                ? {
                    width,
                    height,
                    transform: `translate(calc(-50% + ${x}px), calc(-50% + ${y}px))`
                  }
                : undefined
            }
            onLoad={(event) => rememberImage(event.currentTarget)}
            ref={(node) => {
              if (node?.complete) {
                rememberImage(node);
              }
            }}
            onError={() => setPlaybackError('这张图片打不开')}
            onDoubleClick={(event) => {
              event.preventDefault();
              toggleFit();
            }}
            onPointerDown={onPanStart}
            onPointerMove={onPanMove}
            onPointerUp={() => {
              drag.current = null;
            }}
            onPointerCancel={() => {
              drag.current = null;
            }}
          />
        ) : null}
        {ready && file.kind === 'video' ? (
          // biome-ignore lint/a11y/useMediaCaption: 用户自己的视频没有字幕轨
          <video
            ref={videoRef}
            className={styles.video}
            data-ready={videoFit > 0 ? 'true' : undefined}
            src={src}
            playsInline
            preload="auto"
            style={
              videoFit > 0 && videoSize
                ? {
                    width: videoSize.w * videoFit,
                    height: videoSize.h * videoFit,
                    transform: 'translate(-50%, -50%)'
                  }
                : undefined
            }
            onClick={toggle}
            onPlay={() => setPaused(false)}
            onPause={(event) => {
              setPaused(true);
              writePreviewTime(file.id, event.currentTarget.currentTime);
            }}
            onTimeUpdate={(event) => {
              const time = event.currentTarget.currentTime;
              writePreviewTime(file.id, time);
              setCurrent(time);
            }}
            onProgress={(event) => setBuffered(bufferedEnd(event.currentTarget))}
            onError={() => setPlaybackError('这个视频打不开')}
          />
        ) : null}
        {ready && file.kind === 'video' && paused && !armed ? (
          <IconButton className={clsx(styles.tool, styles.play)} label="播放" onClick={toggle}>
            <PlayIcon weight="fill" />
          </IconButton>
        ) : null}
        {spot.prev ? (
          <IconButton className={clsx(styles.tool, styles.side, styles.prev)} label="上一个" onClick={() => step(-1)}>
            <CaretLeftIcon weight="regular" />
          </IconButton>
        ) : null}
        {spot.next ? (
          <IconButton className={clsx(styles.tool, styles.side, styles.next)} label="下一个" onClick={() => step(1)}>
            <CaretRightIcon weight="regular" />
          </IconButton>
        ) : null}
        {ready && file.kind === 'video' ? (
          <div className={styles.dock}>
            <IconButton
              className={clsx(styles.tool, styles.dockPlay)}
              label={paused ? '播放' : '暂停'}
              size="compact"
              onClick={toggle}
            >
              {paused ? <PlayIcon className={styles.nudge} weight="fill" /> : <PauseIcon weight="fill" />}
            </IconButton>
            <span className={styles.clock}>{clock(current)}</span>
            <Scrub
              className={styles.dockScrub}
              invert
              current={current}
              duration={duration}
              buffered={buffered}
              onSeek={seek}
            />
            <span className={styles.clock}>{clock(duration)}</span>
            <IconButton
              className={styles.tool}
              label={silent ? '取消静音' : '静音'}
              size="compact"
              onClick={() => {
                if (silent) {
                  onMuted(false);
                  if (volume === 0) {
                    onVolume(1);
                  }
                  return;
                }
                onMuted(true);
              }}
            >
              {silent ? <SpeakerSlashIcon weight="regular" /> : <SpeakerHighIcon weight="regular" />}
            </IconButton>
            <input
              className={styles.volume}
              type="range"
              min={0}
              max={1}
              step={0.05}
              value={silent ? 0 : volume}
              aria-label="音量"
              onChange={(event) => {
                const next = Number(event.target.value);
                onVolume(next);
                onMuted(next === 0);
              }}
            />
          </div>
        ) : null}
      </div>
    </>
  );
}
