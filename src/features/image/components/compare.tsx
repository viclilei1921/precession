import { MagnifyingGlassMinusIcon, MagnifyingGlassPlusIcon, XIcon } from '@phosphor-icons/react';
import type { PointerEvent as ReactPointerEvent, SyntheticEvent } from 'react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { fileSizes } from '@/bridge';
import type { ImageQuality } from '@/bridge/task';
import { viewerDirectSrc, viewerDisplay } from '@/bridge/viewer';
import { IconButton } from '@/components/button/icon';
import { SegmentButton } from '@/components/button/segment';
import { formatBytes } from '@/features/files/text';
import { useBackLayer } from '@/layout/mobile/back';
import styles from './compare.module.css';

export type CompareTarget = {
  input: string;
  output: string;
  title: string;
  quality: ImageQuality | null;
};

/** 浏览器能直接显示的地址。JXL 要先在侧车解码。 */
async function imageSrc(path: string) {
  const dot = path.lastIndexOf('.');
  const extension = dot >= 0 ? path.slice(dot + 1).toLowerCase() : '';
  if (extension === 'jxl') {
    const display = await viewerDisplay(path);
    return viewerDirectSrc(display.displayPath);
  }
  return viewerDirectSrc(path);
}

type Zoom = {
  scale: number;
  x: number;
  y: number;
};

const MIN_SCALE = 0.02;
const MAX_SCALE = 16;

function onMac() {
  return /mac/i.test(navigator.userAgent);
}

/** 触控板捏合在 Windows 上是 Ctrl+滚轮，在 macOS 上是 Command+滚轮 */
function wheelZooms(event: WheelEvent) {
  return onMac() ? event.metaKey : event.ctrlKey;
}

/** 滚轮步进。鼠标一格的 delta 通常不小于 100，触摸板捏合更碎。 */
function wheelZoomDelta(deltaY: number) {
  return deltaY / (Math.abs(deltaY) >= 100 ? 1000 : 100);
}

export function CompareView({ target, onClose }: { target: CompareTarget; onClose: () => void }) {
  const { t } = useTranslation('image');
  // 对比浮层是一层返回：标题栏/系统返回先关对比，再退路由
  useBackLayer(true, onClose);
  const stageRef = useRef<HTMLDivElement>(null);
  const [originalSrc, setOriginalSrc] = useState<string | null>(null);
  const [convertedSrc, setConvertedSrc] = useState<string | null>(null);
  const [natural, setNatural] = useState<{ width: number; height: number } | null>(null);
  const [sizes, setSizes] = useState<{ input: number | null; output: number | null }>({ input: null, output: null });
  const [pos, setPos] = useState(50);
  const [zoom, setZoom] = useState<Zoom>({ scale: 1, x: 0, y: 0 });
  const [mode, setMode] = useState<'fit' | 'pixel'>('fit');
  /** 是否已经按当前尺寸套用过适应窗口 */
  const fittedRef = useRef(false);
  /** 缩放和平移都从这里读最新值，避免闭包里的旧状态 */
  const zoomRef = useRef(zoom);
  zoomRef.current = zoom;
  const gestureBase = useRef(1);

  const closeRef = useRef(onClose);
  closeRef.current = onClose;

  useEffect(() => {
    let alive = true;
    void imageSrc(target.input).then((src) => {
      if (alive) {
        setOriginalSrc(src);
      }
    });
    void imageSrc(target.output).then((src) => {
      if (alive) {
        setConvertedSrc(src);
      }
    });
    void fileSizes([target.input, target.output])
      .then((found) => {
        if (!alive) {
          return;
        }
        setSizes({
          input: found.find((item) => item.path === target.input)?.size ?? null,
          output: found.find((item) => item.path === target.output)?.size ?? null
        });
      })
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [target.input, target.output]);

  /** 以给定的缩放倍率把画面摆到舞台中央。倍率省略时按包含整图计算。 */
  const centerAt = useCallback(
    (scale?: number) => {
      const stage = stageRef.current;
      if (!stage || !natural) {
        return;
      }
      const rect = stage.getBoundingClientRect();
      const value = scale ?? Math.min(rect.width / natural.width, rect.height / natural.height);
      const next = {
        scale: value,
        x: (rect.width - natural.width * value) / 2,
        y: (rect.height - natural.height * value) / 2
      };
      zoomRef.current = next;
      setZoom(next);
      setMode(scale == null ? 'fit' : 'pixel');
    },
    [natural]
  );

  /** 图片就位后套用一次适应窗口。 */
  useEffect(() => {
    if (natural && !fittedRef.current) {
      fittedRef.current = true;
      centerAt();
    }
  }, [natural, centerAt]);

  /** 窗口大小变化时，处于适应模式就重新居中。 */
  useEffect(() => {
    if (mode !== 'fit') {
      return;
    }
    function onResize() {
      if (fittedRef.current) {
        centerAt();
      }
    }
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, [mode, centerAt]);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (event.key === 'Escape' || event.key === 'q' || event.key === 'Q') {
        event.preventDefault();
        closeRef.current();
      }
    }
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, []);

  /** 以舞台中心为中心缩放到指定倍率。 */
  const zoomTo = useCallback((value: number) => {
    const stage = stageRef.current;
    if (!stage) {
      return;
    }
    const rect = stage.getBoundingClientRect();
    const current = zoomRef.current;
    const scale = Math.min(MAX_SCALE, Math.max(MIN_SCALE, value));
    const k = scale / current.scale;
    const px = rect.width / 2;
    const py = rect.height / 2;
    const next = { scale, x: px - (px - current.x) * k, y: py - (py - current.y) * k };
    zoomRef.current = next;
    setZoom(next);
    setMode('pixel');
  }, []);

  /** 以舞台中心为中心缩放固定倍率。 */
  const zoomBy = useCallback(
    (factor: number) => {
      zoomTo(zoomRef.current.scale * factor);
    },
    [zoomTo]
  );

  /** 滚轮与其他查看器同构：普通滚轮平移，捏合（Ctrl/Command+滚轮）缩放。
   *  React 的 onWheel 是 passive 的，这里自己挂非 passive 监听。 */
  useEffect(() => {
    const stage = stageRef.current;
    if (!stage) {
      return;
    }
    function onWheel(event: WheelEvent) {
      if (!stage) {
        return;
      }
      event.preventDefault();
      const rect = stage.getBoundingClientRect();
      const current = zoomRef.current;
      if (wheelZooms(event)) {
        const scale = Math.min(MAX_SCALE, Math.max(MIN_SCALE, current.scale - wheelZoomDelta(event.deltaY)));
        const k = scale / current.scale;
        const px = event.clientX - rect.left;
        const py = event.clientY - rect.top;
        const next = { scale, x: px - (px - current.x) * k, y: py - (py - current.y) * k };
        zoomRef.current = next;
        setZoom(next);
        setMode('pixel');
        return;
      }
      const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? rect.height : 1;
      const dx = (event.shiftKey ? event.deltaY : event.deltaX) * unit;
      const dy = (event.shiftKey ? 0 : event.deltaY) * unit;
      const next = { ...current, x: current.x - dx, y: current.y - dy };
      zoomRef.current = next;
      setZoom(next);
      setMode('pixel');
    }
    stage.addEventListener('wheel', onWheel, { passive: false });
    return () => stage.removeEventListener('wheel', onWheel);
  }, []);

  /** macOS 触控板捏合（gesture 事件），与其他查看器一致。 */
  useEffect(() => {
    const stage = stageRef.current;
    if (!stage) {
      return;
    }
    const onStart = (event: Event) => {
      event.preventDefault();
      gestureBase.current = zoomRef.current.scale;
    };
    const onChange = (event: Event) => {
      event.preventDefault();
      const scale = 'scale' in event && typeof event.scale === 'number' ? event.scale : 1;
      zoomTo(gestureBase.current * scale);
    };
    stage.addEventListener('gesturestart', onStart);
    stage.addEventListener('gesturechange', onChange);
    return () => {
      stage.removeEventListener('gesturestart', onStart);
      stage.removeEventListener('gesturechange', onChange);
    };
  }, [zoomTo]);

  function onImageLoad(event: SyntheticEvent<HTMLImageElement>) {
    const image = event.currentTarget;
    setNatural({ width: image.naturalWidth, height: image.naturalHeight });
  }

  // 空白处拖动平移；双指捏合缩放，与其他查看器一致。
  const pointers = useRef(new Map<number, { x: number; y: number }>());
  const panRef = useRef<{ id: number; x: number; y: number; ox: number; oy: number } | null>(null);
  const pinchRef = useRef<{ dist: number; scale: number; x: number; y: number; mid: { x: number; y: number } } | null>(
    null
  );
  const dividerRef = useRef<HTMLDivElement>(null);

  function beginPinch() {
    const points = [...pointers.current.values()];
    if (points.length < 2) {
      return null;
    }
    const current = zoomRef.current;
    const mid = { x: (points[0].x + points[1].x) / 2, y: (points[0].y + points[1].y) / 2 };
    return {
      dist: Math.hypot(points[0].x - points[1].x, points[0].y - points[1].y) || 1,
      scale: current.scale,
      x: current.x,
      y: current.y,
      mid
    };
  }

  function onStagePointerDown(event: ReactPointerEvent) {
    pointers.current.set(event.pointerId, { x: event.clientX, y: event.clientY });
    if (pointers.current.size >= 2) {
      panRef.current = null;
      pinchRef.current = beginPinch();
      return;
    }
    if ((event.target as HTMLElement).closest(`.${styles.handle}`)) {
      return;
    }
    panRef.current = {
      id: event.pointerId,
      x: event.clientX,
      y: event.clientY,
      ox: zoomRef.current.x,
      oy: zoomRef.current.y
    };
    event.currentTarget.setPointerCapture(event.pointerId);
  }

  function onStagePointerMove(event: ReactPointerEvent) {
    const pan = panRef.current;
    if (pan && pan.id === event.pointerId) {
      const next = { ...zoomRef.current, x: pan.ox + event.clientX - pan.x, y: pan.oy + event.clientY - pan.y };
      zoomRef.current = next;
      setZoom(next);
      setMode('pixel');
      return;
    }
    if (dividerRef.current?.hasPointerCapture(event.pointerId)) {
      const stage = stageRef.current;
      if (!stage) {
        return;
      }
      const rect = stage.getBoundingClientRect();
      const percent = ((event.clientX - rect.left) / rect.width) * 100;
      setPos(Math.min(100, Math.max(0, percent)));
      return;
    }
    if (!pointers.current.has(event.pointerId)) {
      return;
    }
    pointers.current.set(event.pointerId, { x: event.clientX, y: event.clientY });
    const pinch = pinchRef.current;
    if (pointers.current.size >= 2 && pinch) {
      const points = [...pointers.current.values()];
      const dist = Math.hypot(points[0].x - points[1].x, points[0].y - points[1].y) || 1;
      const mid = { x: (points[0].x + points[1].x) / 2, y: (points[0].y + points[1].y) / 2 };
      const scale = Math.min(MAX_SCALE, Math.max(MIN_SCALE, pinch.scale * (dist / pinch.dist)));
      const k = scale / pinch.scale;
      const next = {
        scale,
        x: mid.x - (pinch.mid.x - pinch.x) * k,
        y: mid.y - (pinch.mid.y - pinch.y) * k
      };
      zoomRef.current = next;
      setZoom(next);
      setMode('pixel');
    }
  }

  function onStagePointerUp(event: ReactPointerEvent) {
    pointers.current.delete(event.pointerId);
    if (pointers.current.size < 2) {
      pinchRef.current = null;
    }
    if (panRef.current?.id === event.pointerId) {
      panRef.current = null;
    }
    // 捏合结束后剩一根手指，接续为拖动平移。
    if (pointers.current.size === 1 && !panRef.current) {
      const entry = [...pointers.current.entries()][0];
      if (entry) {
        panRef.current = { id: entry[0], x: entry[1].x, y: entry[1].y, ox: zoomRef.current.x, oy: zoomRef.current.y };
      }
    }
    dividerRef.current?.releasePointerCapture(event.pointerId);
  }

  function onDividerPointerDown(event: ReactPointerEvent) {
    event.stopPropagation();
    dividerRef.current?.setPointerCapture(event.pointerId);
  }

  const transform = `translate(${zoom.x}px, ${zoom.y}px) scale(${zoom.scale})`;
  const transformStyle = { transform, width: natural?.width ?? 0, height: natural?.height ?? 0 } as const;

  const image = (src: string | null, alt: string) =>
    src ? <img className={styles.image} src={src} alt={alt} draggable={false} onLoad={onImageLoad} /> : null;

  return (
    <div className={styles.view} role="dialog" aria-modal="true" aria-label={t('compare.aria')}>
      <div
        ref={stageRef}
        className={styles.stage}
        role="application"
        aria-label={t('compare.aria')}
        onPointerDown={onStagePointerDown}
        onPointerMove={onStagePointerMove}
        onPointerUp={onStagePointerUp}
        onPointerCancel={onStagePointerUp}
        onDoubleClick={mode === 'fit' ? () => centerAt(1) : () => centerAt()}
      >
        <div className={styles.layer}>
          <div className={styles.canvas} style={transformStyle}>
            {image(originalSrc, t('compare.original'))}
          </div>
        </div>
        <div className={styles.layer} style={{ clipPath: `inset(0 0 0 ${pos}%)` }}>
          <div className={styles.canvas} style={transformStyle}>
            {image(convertedSrc, t('compare.converted'))}
          </div>
        </div>
        <div className={styles.divider} style={{ left: `${pos}%` }}>
          <div
            ref={dividerRef}
            className={styles.handle}
            role="slider"
            aria-label={t('compare.divider')}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={Math.round(pos)}
            tabIndex={0}
            onPointerDown={onDividerPointerDown}
            onKeyDown={(event) => {
              if (event.key === 'ArrowLeft') {
                setPos((current) => Math.max(0, current - 2));
              } else if (event.key === 'ArrowRight') {
                setPos((current) => Math.min(100, current + 2));
              }
            }}
          />
        </div>
        <span className={styles.tag} data-side="left">
          {t('compare.original')}
        </span>
        <span className={styles.tag} data-side="right">
          {t('compare.converted')}
        </span>
      </div>
      <div className={styles.dock}>
        <div className={styles.info}>
          <span className={styles.title}>{target.title}</span>
          <span className={styles.size}>
            {sizes.input != null ? formatBytes(sizes.input) : '—'}
            {' → '}
            {sizes.output != null ? formatBytes(sizes.output) : '—'}
          </span>
          {target.quality?.score != null && target.quality.band ? (
            <span className={styles.quality}>
              {t('quality.score', { score: target.quality.score.toFixed(1) })}
              {' · '}
              {t(`quality.bands.${target.quality.band}`)}
            </span>
          ) : target.quality?.problem ? (
            <span className={styles.quality}>{t(`quality.problems.${target.quality.problem}`)}</span>
          ) : null}
        </div>
        <div className={styles.spacer} />
        <div className={styles.group}>
          <SegmentButton className={styles.seg} selected={mode === 'fit'} onClick={() => centerAt()}>
            {t('compare.fit')}
          </SegmentButton>
          <SegmentButton className={styles.seg} selected={mode === 'pixel'} onClick={() => centerAt(1)}>
            {t('compare.pixel')}
          </SegmentButton>
        </div>
        <IconButton label={t('compare.zoomOut')} size="compact" onClick={() => zoomBy(1 / 1.2)}>
          <MagnifyingGlassMinusIcon weight="regular" />
        </IconButton>
        <span className={styles.percent}>{Math.round(zoom.scale * 100)}%</span>
        <IconButton label={t('compare.zoomIn')} size="compact" onClick={() => zoomBy(1.2)}>
          <MagnifyingGlassPlusIcon weight="regular" />
        </IconButton>
        <div className={styles.spacer} />
        <IconButton label={t('compare.close')} size="compact" onClick={onClose}>
          <XIcon weight="regular" />
        </IconButton>
      </div>
    </div>
  );
}
