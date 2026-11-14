import type { PointerEvent as ReactPointerEvent } from 'react';
import { useEffect, useRef, useState } from 'react';
import styles from './stage.module.css';
import { createWheelPull, pullThreshold } from './wheel-paging';

type Point = { x: number; y: number };
type Size = { width: number; height: number };

type StageProps = {
  src: string;
  alt: string;
  /** `fit` 适应窗口。数字是相对原始像素的比例，1 为实际大小 */
  scale: 'fit' | number;
  /** 下拉框或实际大小按钮换比例时递增，用来把画面拉回中心 */
  panReset: number;
  onUserScale: (scale: number) => void;
  onNatural: (size: Size) => void;
  /** 当前生效比例变化时上报（进入 fit、窗口缩放等）。可选 */
  onScale?: (scale: number) => void;
  /** 舞台内双击。看图/文件查看用它切换 适应/实际大小。可选 */
  onDoubleClick?: (event: React.MouseEvent<HTMLDivElement>) => void;
  /**
   * 缩放不超过适应窗口时，纵向滚动带动整张图，超过阈值翻页。
   * 返回 false 表示这个方向没有上一张或下一张。
   */
  onWheelPage?: (direction: 1 | -1) => boolean;
};

const MIN_SCALE = 0.1;
const MAX_SCALE = 8;

function onMac() {
  return /mac/i.test(navigator.userAgent);
}

/** 触摸板捏合在 Windows 上是 Ctrl+滚轮，在 macOS 上是 Command+滚轮 */
function wheelZooms(event: WheelEvent) {
  return onMac() ? event.metaKey : event.ctrlKey;
}

/**
 * 滚轮步进。鼠标一格的 delta 通常不小于 100，触摸板捏合更碎。
 * 分母跟着换，小步进才看得动。
 */
function wheelZoomDelta(deltaY: number) {
  return deltaY / (Math.abs(deltaY) >= 100 ? 1000 : 100);
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

function clampPan(pan: Point, view: Size, image: Size): Point {
  const limitX = Math.max(0, (image.width - view.width) / 2);
  const limitY = Math.max(0, (image.height - view.height) / 2);
  return { x: clamp(pan.x, -limitX, limitX), y: clamp(pan.y, -limitY, limitY) };
}

/** 以指针为中心缩放后，画面该挪到哪里 */
function zoomAt(prev: number, next: number, pan: Point, view: Size, pointer: Point): Point {
  if (prev === 0) {
    return pan;
  }
  const ratio = next / prev;
  return {
    x: pan.x + (pointer.x - (view.width / 2 + pan.x)) * (1 - ratio),
    y: pan.y + (pointer.y - (view.height / 2 + pan.y)) * (1 - ratio)
  };
}

/** 看图区域。捏合缩放；放大超过适应窗口时滚轮平移，不超过时滚轮拉动整张图翻页。 */
export function Stage({
  src,
  alt,
  scale,
  panReset,
  onUserScale,
  onNatural,
  onScale,
  onDoubleClick,
  onWheelPage
}: StageProps) {
  const rootRef = useRef<HTMLDivElement>(null);
  const [natural, setNatural] = useState<Size | null>(null);
  const [view, setView] = useState<Size>({ width: 0, height: 0 });
  const [pan, setPan] = useState<Point>({ x: 0, y: 0 });
  /** 不超过适应窗口时，整张图跟着滚轮走的纵向偏移。向下为正。 */
  const [pull, setPull] = useState(0);
  const pullRef = useRef<ReturnType<typeof createWheelPull> | null>(null);
  const pointers = useRef(new Map<number, Point>());
  const drag = useRef<{ id: number; x: number; y: number; pan: Point } | null>(null);
  const pinch = useRef<{ dist: number; scale: number; mid: Point; pan: Point } | null>(null);
  const gestureBase = useRef(1);
  const anchor = useRef<Point | null>(null);
  const onUserScaleRef = useRef(onUserScale);
  const onNaturalRef = useRef(onNatural);
  const onScaleRef = useRef(onScale);
  const onWheelPageRef = useRef(onWheelPage);
  onUserScaleRef.current = onUserScale;
  onNaturalRef.current = onNatural;
  onScaleRef.current = onScale;
  onWheelPageRef.current = onWheelPage;

  const fit =
    natural && view.width > 0 && view.height > 0
      ? Math.min(view.width / natural.width, view.height / natural.height)
      : 1;
  const effective = scale === 'fit' ? fit : scale;
  const stateRef = useRef({ pan, effective, fit, natural, view });
  stateRef.current = { pan, effective, fit, natural, view };

  useEffect(() => {
    onScaleRef.current?.(effective);
  }, [effective]);

  useEffect(() => {
    void panReset;
    setPan({ x: 0, y: 0 });
    pullRef.current?.reset();
  }, [panReset]);

  useEffect(() => {
    const node = rootRef.current;
    if (!node) {
      return;
    }
    const update = () => setView({ width: node.clientWidth, height: node.clientHeight });
    update();
    const observer = new ResizeObserver(update);
    observer.observe(node);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const node = rootRef.current;
    if (!node) {
      return;
    }

    const zoomBy = (next: number, pointer: Point) => {
      const current = stateRef.current;
      if (!current.natural || current.view.width === 0) {
        return;
      }
      const image = { width: current.natural.width * next, height: current.natural.height * next };
      setPan(clampPan(zoomAt(current.effective, next, current.pan, current.view, pointer), current.view, image));
      onUserScaleRef.current(next);
    };

    const covers = (event: WheelEvent) => {
      const rect = node.getBoundingClientRect();
      return (
        event.clientX >= rect.left &&
        event.clientX <= rect.right &&
        event.clientY >= rect.top &&
        event.clientY <= rect.bottom
      );
    };

    const pointerInView = (clientX: number, clientY: number) => {
      const rect = node.getBoundingClientRect();
      return { x: clientX - rect.left, y: clientY - rect.top };
    };

    const pull = createWheelPull({
      onChange: setPull,
      step: (direction) => onWheelPageRef.current?.(direction) !== false
    });
    pullRef.current = pull;

    const onPinchWheel = (event: WheelEvent) => {
      if (!wheelZooms(event)) {
        return;
      }
      event.preventDefault();
      const current = stateRef.current;
      if (!current.natural || current.view.width === 0) {
        return;
      }
      pull.reset();
      const next = clamp(current.effective - wheelZoomDelta(event.deltaY), MIN_SCALE, MAX_SCALE);
      const pointer = covers(event)
        ? pointerInView(event.clientX, event.clientY)
        : (anchor.current ?? { x: current.view.width / 2, y: current.view.height / 2 });
      anchor.current = pointer;
      zoomBy(next, pointer);
    };

    const onWheel = (event: WheelEvent) => {
      if (wheelZooms(event)) {
        return;
      }
      if (event.target instanceof Element && event.target.closest('input, textarea, select')) {
        return;
      }
      const current = stateRef.current;
      if (!current.natural || current.view.width === 0) {
        return;
      }
      event.preventDefault();
      const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? current.view.height : 1;
      const atMostFit = current.effective <= current.fit + 0.001;
      if (atMostFit && onWheelPageRef.current && !event.shiftKey && Math.abs(event.deltaY) > Math.abs(event.deltaX)) {
        if (current.pan.x !== 0 || current.pan.y !== 0) {
          setPan({ x: 0, y: 0 });
        }
        pull.move(event.deltaY * unit, pullThreshold(current.view.height));
        return;
      }
      const dx = (event.shiftKey ? event.deltaY : event.deltaX) * unit;
      const dy = (event.shiftKey ? 0 : event.deltaY) * unit;
      const image = {
        width: current.natural.width * current.effective,
        height: current.natural.height * current.effective
      };
      setPan(clampPan({ x: current.pan.x - dx, y: current.pan.y - dy }, current.view, image));
    };

    const onGestureStart = (event: Event) => {
      event.preventDefault();
      pull.reset();
      gestureBase.current = stateRef.current.effective;
    };
    const onGestureChange = (event: Event) => {
      event.preventDefault();
      const scale = 'scale' in event && typeof event.scale === 'number' ? event.scale : 1;
      const current = stateRef.current;
      zoomBy(clamp(gestureBase.current * scale, MIN_SCALE, MAX_SCALE), {
        x: current.view.width / 2,
        y: current.view.height / 2
      });
    };

    // 挂在父级上，工具条和左右切换也算在这片看图区域里
    const scope = node.parentElement ?? node;
    window.addEventListener('wheel', onPinchWheel, { capture: true, passive: false });
    scope.addEventListener('wheel', onWheel, { passive: false });
    node.addEventListener('gesturestart', onGestureStart);
    node.addEventListener('gesturechange', onGestureChange);
    return () => {
      pull.dispose();
      pullRef.current = null;
      window.removeEventListener('wheel', onPinchWheel, { capture: true });
      scope.removeEventListener('wheel', onWheel);
      node.removeEventListener('gesturestart', onGestureStart);
      node.removeEventListener('gesturechange', onGestureChange);
    };
  }, []);

  useEffect(() => {
    if (effective > fit + 0.001) {
      pullRef.current?.reset();
    }
  }, [effective, fit]);

  function localPoint(clientX: number, clientY: number) {
    const rect = rootRef.current?.getBoundingClientRect();
    if (!rect) {
      return { x: clientX, y: clientY };
    }
    return { x: clientX - rect.left, y: clientY - rect.top };
  }

  function beginPinch() {
    const points = [...pointers.current.values()];
    if (points.length < 2) {
      return null;
    }
    const current = stateRef.current;
    const mid = localPoint((points[0].x + points[1].x) / 2, (points[0].y + points[1].y) / 2);
    return {
      dist: Math.hypot(points[0].x - points[1].x, points[0].y - points[1].y) || 1,
      scale: current.effective,
      mid,
      pan: current.pan
    };
  }

  function releaseCapture(target: HTMLDivElement) {
    for (const id of pointers.current.keys()) {
      if (target.hasPointerCapture(id)) {
        target.releasePointerCapture(id);
      }
    }
  }

  function onPointerDown(event: ReactPointerEvent<HTMLDivElement>) {
    pullRef.current?.reset();
    pointers.current.set(event.pointerId, { x: event.clientX, y: event.clientY });
    if (pointers.current.size >= 2) {
      releaseCapture(event.currentTarget);
      drag.current = null;
      pinch.current = beginPinch();
      return;
    }
    drag.current = { id: event.pointerId, x: event.clientX, y: event.clientY, pan: stateRef.current.pan };
  }

  function onPointerMove(event: ReactPointerEvent<HTMLDivElement>) {
    if (!pointers.current.has(event.pointerId)) {
      return;
    }
    pointers.current.set(event.pointerId, { x: event.clientX, y: event.clientY });
    anchor.current = localPoint(event.clientX, event.clientY);
    if (pointers.current.size === 1 && drag.current?.id === event.pointerId) {
      if (!event.currentTarget.hasPointerCapture(event.pointerId)) {
        event.currentTarget.setPointerCapture(event.pointerId);
      }
    }
    const current = stateRef.current;
    if (!current.natural) {
      return;
    }
    if (pointers.current.size >= 2 && pinch.current) {
      const points = [...pointers.current.values()];
      const dist = Math.hypot(points[0].x - points[1].x, points[0].y - points[1].y) || 1;
      const next = clamp(pinch.current.scale * (dist / pinch.current.dist), MIN_SCALE, MAX_SCALE);
      const mid = localPoint((points[0].x + points[1].x) / 2, (points[0].y + points[1].y) / 2);
      const shifted = zoomAt(pinch.current.scale, next, pinch.current.pan, current.view, pinch.current.mid);
      const panNext = { x: shifted.x + (mid.x - pinch.current.mid.x), y: shifted.y + (mid.y - pinch.current.mid.y) };
      const image = { width: current.natural.width * next, height: current.natural.height * next };
      setPan(clampPan(panNext, current.view, image));
      onUserScaleRef.current(next);
      return;
    }
    const dragging = drag.current;
    if (!dragging || dragging.id !== event.pointerId) {
      return;
    }
    const image = {
      width: current.natural.width * current.effective,
      height: current.natural.height * current.effective
    };
    setPan(
      clampPan(
        { x: dragging.pan.x + (event.clientX - dragging.x), y: dragging.pan.y + (event.clientY - dragging.y) },
        current.view,
        image
      )
    );
  }

  function onPointerEnd(event: ReactPointerEvent<HTMLDivElement>) {
    pointers.current.delete(event.pointerId);
    if (pointers.current.size < 2) {
      pinch.current = null;
    }
    if (drag.current?.id === event.pointerId) {
      drag.current = null;
    }
    if (pointers.current.size === 1) {
      const entry = [...pointers.current.entries()][0];
      if (entry) {
        drag.current = { id: entry[0], x: entry[1].x, y: entry[1].y, pan: stateRef.current.pan };
      }
    }
  }

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: 舞台本身承载指针/滚轮/双击交互，内部 img 不可交互
    <div
      ref={rootRef}
      className={styles.stage}
      onDoubleClick={onDoubleClick}
      // 双击的第二下按住时不让浏览器选字，避免看图时出现选中框
      onMouseDown={(event) => {
        if (event.detail >= 2) {
          event.preventDefault();
        }
      }}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerEnd}
      onPointerCancel={onPointerEnd}
    >
      <img
        className={styles.image}
        src={src}
        alt={alt}
        draggable={false}
        style={
          natural
            ? {
                width: natural.width,
                height: natural.height,
                transform: `translate(${pan.x}px, ${pan.y + pull}px) scale(${effective})`
              }
            : { visibility: 'hidden' }
        }
        onLoad={(event) => {
          const size = { width: event.currentTarget.naturalWidth, height: event.currentTarget.naturalHeight };
          if (size.width > 0 && size.height > 0) {
            setNatural(size);
            onNaturalRef.current(size);
          }
        }}
      />
    </div>
  );
}
