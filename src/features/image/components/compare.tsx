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

export function CompareView({ target, onClose }: { target: CompareTarget; onClose: () => void }) {
  const { t } = useTranslation('image');
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
      if (event.key === 'Escape') {
        event.preventDefault();
        closeRef.current();
      }
    }
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, []);

  /** 滚轮缩放。React 的 onWheel 是 passive 的，这里自己挂非 passive 监听。 */
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
      const factor = event.deltaY < 0 ? 1.2 : 1 / 1.2;
      const current = zoomRef.current;
      const scale = Math.min(MAX_SCALE, Math.max(MIN_SCALE, current.scale * factor));
      const k = scale / current.scale;
      const px = event.clientX - rect.left;
      const py = event.clientY - rect.top;
      const next = { scale, x: px - (px - current.x) * k, y: py - (py - current.y) * k };
      zoomRef.current = next;
      setZoom(next);
      setMode('pixel');
    }
    stage.addEventListener('wheel', onWheel, { passive: false });
    return () => stage.removeEventListener('wheel', onWheel);
  }, []);

  /** 以舞台中心为中心缩放固定倍率。 */
  const zoomBy = useCallback((factor: number) => {
    const stage = stageRef.current;
    if (!stage) {
      return;
    }
    const rect = stage.getBoundingClientRect();
    const current = zoomRef.current;
    const scale = Math.min(MAX_SCALE, Math.max(MIN_SCALE, current.scale * factor));
    const k = scale / current.scale;
    const px = rect.width / 2;
    const py = rect.height / 2;
    const next = { scale, x: px - (px - current.x) * k, y: py - (py - current.y) * k };
    zoomRef.current = next;
    setZoom(next);
    setMode('pixel');
  }, []);

  function onImageLoad(event: SyntheticEvent<HTMLImageElement>) {
    const image = event.currentTarget;
    setNatural({ width: image.naturalWidth, height: image.naturalHeight });
  }

  // 空白处拖动平移。
  const panRef = useRef<{ x: number; y: number; ox: number; oy: number } | null>(null);
  const dividerRef = useRef<HTMLDivElement>(null);

  function onStagePointerDown(event: ReactPointerEvent) {
    if ((event.target as HTMLElement).closest(`.${styles.handle}`)) {
      return;
    }
    panRef.current = { x: event.clientX, y: event.clientY, ox: zoomRef.current.x, oy: zoomRef.current.y };
    event.currentTarget.setPointerCapture(event.pointerId);
  }

  function onStagePointerMove(event: ReactPointerEvent) {
    const pan = panRef.current;
    if (pan) {
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
    }
  }

  function onStagePointerUp(event: ReactPointerEvent) {
    panRef.current = null;
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
