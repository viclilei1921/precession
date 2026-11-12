import type { PointerEvent as ReactPointerEvent } from 'react';
import { useEffect, useRef, useState } from 'react';
import styles from './page.module.css';

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

/** 看图区域。触控板捏合和双指缩放，滚轮和拖移只平移。 */
export function Stage({ src, alt, scale, panReset, onUserScale, onNatural }: StageProps) {
  const rootRef = useRef<HTMLDivElement>(null);
  const [natural, setNatural] = useState<Size | null>(null);
  const [view, setView] = useState<Size>({ width: 0, height: 0 });
  const [pan, setPan] = useState<Point>({ x: 0, y: 0 });
  const pointers = useRef(new Map<number, Point>());
  const drag = useRef<{ id: number; x: number; y: number; pan: Point } | null>(null);
  const pinch = useRef<{ dist: number; scale: number; mid: Point; pan: Point } | null>(null);
  const gestureBase = useRef(1);
  const anchor = useRef<Point | null>(null);
  const onUserScaleRef = useRef(onUserScale);
  const onNaturalRef = useRef(onNatural);
  onUserScaleRef.current = onUserScale;
  onNaturalRef.current = onNatural;

  const fit =
    natural && view.width > 0 && view.height > 0
      ? Math.min(view.width / natural.width, view.height / natural.height)
      : 1;
  const effective = scale === 'fit' ? fit : scale;
  const stateRef = useRef({ pan, effective, natural, view });
  stateRef.current = { pan, effective, natural, view };

  useEffect(() => {
    void panReset;
    setPan({ x: 0, y: 0 });
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

    const onPinchWheel = (event: WheelEvent) => {
      if (!wheelZooms(event)) {
        return;
      }
      event.preventDefault();
      const current = stateRef.current;
      if (!current.natural || current.view.width === 0) {
        return;
      }
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
      const current = stateRef.current;
      if (!current.natural || current.view.width === 0) {
        return;
      }
      event.preventDefault();
      const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? current.view.height : 1;
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

    window.addEventListener('wheel', onPinchWheel, { capture: true, passive: false });
    node.addEventListener('wheel', onWheel, { passive: false });
    node.addEventListener('gesturestart', onGestureStart);
    node.addEventListener('gesturechange', onGestureChange);
    return () => {
      window.removeEventListener('wheel', onPinchWheel, { capture: true });
      node.removeEventListener('wheel', onWheel);
      node.removeEventListener('gesturestart', onGestureStart);
      node.removeEventListener('gesturechange', onGestureChange);
    };
  }, []);

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
    <div
      ref={rootRef}
      className={styles.stage}
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
                transform: `translate(${pan.x}px, ${pan.y}px) scale(${effective})`
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
