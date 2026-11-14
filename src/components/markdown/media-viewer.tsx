import { CaretLeftIcon, CaretRightIcon, MinusIcon, PlusIcon, XIcon } from '@phosphor-icons/react';
import clsx from 'clsx';
import type { PointerEvent as ReactPointerEvent } from 'react';
import { useEffect, useState } from 'react';
import { createPortal } from 'react-dom';
import { useTranslation } from 'react-i18next';
import { IconButton } from '@/components/button/icon';
import { Stage } from '@/components/stage';
import overlay from '@/components/stage/overlay.module.css';
import { useBackLayer } from '@/layout/mobile/back';
import { useAppStore } from '@/store/app';

export type MediaItem = {
  kind: 'image' | 'video';
  src: string;
  caption?: string;
};

type MediaViewerProps = {
  items: MediaItem[];
  index: number;
  onClose: () => void;
};

/** 全屏媒体查看。
 *  桌面端与「看图」同构：底部悬浮工具条 + 缩放舞台（同一 Stage 组件）。
 *  手机端参考微信：纯黑、点按显隐控件、左右滑动切换（双指捏合由 Stage 支持）。
 *  快捷键：Esc/q 关闭，←/→ 切换，0 适应窗口，1 实际大小。
 *  缩放不超过适应窗口时，滚轮带动整张图，超过阈值再切换。 */
export function MediaViewer({ items, index, onClose }: MediaViewerProps) {
  const { t } = useTranslation('ui');
  // 全屏预览是一层返回：标题栏/系统返回先关预览，再退路由
  useBackLayer(true, onClose);
  const mobile = useAppStore((state) => state.isMobile());
  const [shown, setShown] = useState(false);
  const [current, setCurrent] = useState(index);
  const [chrome, setChrome] = useState(true);
  const item = items[current];

  // 图片缩放状态（与看图同构）。切换图片时整体重置
  const [mode, setMode] = useState<'fit' | 'free'>('fit');
  const [zoom, setZoom] = useState(1);
  const [effective, setEffective] = useState(1);
  const [natural, setNatural] = useState<{ w: number; h: number } | null>(null);
  const [panReset, setPanReset] = useState(0);

  useEffect(() => {
    const frame = requestAnimationFrame(() => setShown(true));
    return () => cancelAnimationFrame(frame);
  }, []);

  // biome-ignore lint/correctness/useExhaustiveDependencies: 切图时重置缩放，只在 current 变化时跑
  useEffect(() => {
    setMode('fit');
    setZoom(1);
    setEffective(1);
    setNatural(null);
    setPanReset((value) => value + 1);
  }, [current]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const target = event.target;
      if (target instanceof HTMLElement && target.closest('input, textarea, select')) {
        return;
      }
      if (event.key === 'Escape' || event.key === 'q' || event.key === 'Q') {
        onClose();
        return;
      }
      if (event.key === 'ArrowLeft' && items.length > 1) {
        setCurrent((value) => (value - 1 + items.length) % items.length);
      }
      if (event.key === 'ArrowRight' && items.length > 1) {
        setCurrent((value) => (value + 1) % items.length);
      }
      if (item?.kind === 'image') {
        if (event.key === '0') {
          setMode('fit');
          setPanReset((value) => value + 1);
        } else if (event.key === '1') {
          setMode('free');
          setZoom(1);
          setPanReset((value) => value + 1);
        }
      }
    };
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, [items.length, onClose, item?.kind]);

  if (!item) {
    return null;
  }

  const isImage = item.kind === 'image';
  const go = (step: number) => setCurrent((value) => (value + step + items.length) % items.length);

  function toggleFit() {
    if (mode === 'fit') {
      setMode('free');
      setZoom(1);
      setPanReset((value) => value + 1);
      return;
    }
    setMode('fit');
    setPanReset((value) => value + 1);
  }

  function zoomBy(factor: number) {
    setMode('free');
    setZoom(Math.min(8, Math.max(0.1, effective * factor)));
  }

  const onPointerDown = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (event.pointerType !== 'touch') {
      return;
    }
    event.currentTarget.dataset.swipe = `${event.clientX}:${event.clientY}`;
  };

  const onPointerUp = (event: ReactPointerEvent<HTMLDivElement>) => {
    const start = event.currentTarget.dataset.swipe;
    delete event.currentTarget.dataset.swipe;
    if (!start || event.pointerType !== 'touch') {
      return;
    }
    const [sx, sy] = start.split(':').map(Number);
    const dx = event.clientX - (sx ?? 0);
    const dy = event.clientY - (sy ?? 0);
    if (Math.abs(dx) > 48 && Math.abs(dx) > Math.abs(dy) * 1.2 && items.length > 1) {
      go(dx < 0 ? 1 : -1);
      return;
    }
    // 轻点切换控件显示（微信风格）
    if (Math.abs(dx) < 10 && Math.abs(dy) < 10 && isImage) {
      setChrome((value) => !value);
    }
  };

  const onStageClick = (event: React.MouseEvent<HTMLDivElement>) => {
    // 空白区域：桌面关闭，手机切控件显隐
    if (event.target !== event.currentTarget) {
      return;
    }
    if (mobile && isImage) {
      setChrome((value) => !value);
      return;
    }
    if (!mobile) {
      onClose();
    }
  };

  return createPortal(
    <div
      className={overlay.overlay}
      data-mobile={mobile ? 'true' : undefined}
      data-chrome={chrome ? 'true' : 'false'}
      data-open={shown ? 'true' : undefined}
      role="dialog"
      aria-modal="true"
      tabIndex={-1}
      onKeyDown={(event) => {
        if (event.key === 'Escape') {
          onClose();
        }
      }}
    >
      {/* biome-ignore lint/a11y/noStaticElementInteractions: 空白区域点击关闭/切控件，键盘由 Esc/q/←/→/0/1 覆盖 */}
      {/* biome-ignore lint/a11y/useKeyWithClickEvents: 键盘已有 Esc/左右箭头监听 */}
      <div
        className={overlay.stageWrap}
        onPointerDown={onPointerDown}
        onPointerUp={onPointerUp}
        onClick={onStageClick}
        onDoubleClick={(event) => {
          if (!mobile && isImage) {
            event.preventDefault();
            toggleFit();
          }
        }}
      >
        {isImage ? (
          <Stage
            key={item.src}
            src={item.src}
            alt={item.caption ?? ''}
            scale={mode === 'fit' ? 'fit' : zoom}
            panReset={panReset}
            onScale={setEffective}
            onUserScale={(scale) => {
              setMode('free');
              setZoom(scale);
            }}
            onNatural={(size) => setNatural({ w: size.width, h: size.height })}
            onWheelPage={(direction) => {
              if (items.length < 2) {
                return false;
              }
              go(direction);
              return true;
            }}
          />
        ) : (
          // biome-ignore lint/a11y/useMediaCaption: 播放的是本地导入的文件，没有字幕轨
          <video className={overlay.video} src={item.src} controls autoPlay playsInline />
        )}
        {items.length > 1 ? (
          <>
            <IconButton
              className={clsx(overlay.side, overlay.prev)}
              label={t('markdown.viewerPrev')}
              onClick={() => go(-1)}
            >
              <CaretLeftIcon weight="regular" />
            </IconButton>
            <IconButton
              className={clsx(overlay.side, overlay.next)}
              label={t('markdown.viewerNext')}
              onClick={() => go(1)}
            >
              <CaretRightIcon weight="regular" />
            </IconButton>
          </>
        ) : null}
        <div className={overlay.bar}>
          {items.length > 1 ? (
            <span className={overlay.chip}>
              {current + 1} / {items.length}
            </span>
          ) : null}
          <p className={overlay.meta} title={item.caption}>
            {item.caption || (isImage ? t('markdown.viewerImage') : t('markdown.viewerVideo'))}
            {natural ? (
              <span className={overlay.sub}>
                {' '}
                · {natural.w} × {natural.h}
              </span>
            ) : null}
          </p>
          <div className={overlay.tools}>
            {!mobile && isImage && natural ? (
              <div className={overlay.zoom}>
                <IconButton
                  label={t('markdown.viewerZoomOut')}
                  size="compact"
                  shape="square"
                  onClick={() => zoomBy(1 / 1.25)}
                >
                  <MinusIcon weight="regular" />
                </IconButton>
                <button type="button" className={overlay.zoomLabel} title={t('markdown.viewerFit')} onClick={toggleFit}>
                  {mode === 'fit' ? t('markdown.viewerFit') : `${Math.round(effective * 100)}%`}
                </button>
                <IconButton
                  label={t('markdown.viewerZoomIn')}
                  size="compact"
                  shape="square"
                  onClick={() => zoomBy(1.25)}
                >
                  <PlusIcon weight="regular" />
                </IconButton>
              </div>
            ) : null}
            <IconButton label={t('markdown.viewerClose')} onClick={onClose}>
              <XIcon weight="regular" />
            </IconButton>
          </div>
        </div>
      </div>
    </div>,
    document.body
  );
}
