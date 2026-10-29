import clsx from 'clsx';
import type { ReactNode, PointerEvent as ReactPointerEvent, Ref } from 'react';
import { useLayoutEffect, useRef, useState } from 'react';
import styles from './index.module.css';

/** 允许滚动的方向。默认两个方向都按内容溢出决定。 */
type ScrollAxis = 'vertical' | 'horizontal' | 'both';

type ScrollProps = {
  /** 根节点类名。高度由调用方限制，组件自己不撑开页面。 */
  className?: string;
  /** 哪个方向可以滚动 */
  axis?: ScrollAxis;
  /**
   * 滑块贴在右侧留白的外缘，不盖住内容。
   * 父级右侧留白默认是 `--space-5`，可用 `--scroll-edge` 改成同一宽度。这条路径上不能再裁切溢出。
   */
  edge?: boolean;
  /** 竖向溢出时在右侧让出滑块宽度，滑块不盖住内容。分栏里用这个。 */
  gutter?: boolean;
  /** 滚到头之后，把剩余滚轮交给外层 */
  chain?: boolean;
  /** 滚动层节点，供外部对齐位置 */
  viewportRef?: Ref<HTMLDivElement>;
  children: ReactNode;
};

function setRef<T>(ref: Ref<T> | undefined, value: T | null) {
  if (typeof ref === 'function') {
    ref(value);
    return;
  }
  if (ref) {
    ref.current = value;
  }
}

type AxisName = 'x' | 'y';

type AxisState = {
  overflow: boolean;
  size: number;
  offset: number;
  travel: number;
  maxScroll: number;
};

type DragState = {
  axis: AxisName;
  startClient: number;
  startScroll: number;
  travel: number;
  maxScroll: number;
};

const HIDE_MS = 700;

const idleAxis: AxisState = {
  overflow: false,
  size: 0,
  offset: 0,
  travel: 0,
  maxScroll: 0
};

function readLength(el: HTMLElement, name: string, fallback: number) {
  const value = Number.parseFloat(getComputedStyle(el).getPropertyValue(name));
  return Number.isFinite(value) ? value : fallback;
}

/** 根据容器和内容尺寸算出滑块。轨道长度和 CSS 里的留白一致。 */
function measureAxis(
  scroll: number,
  client: number,
  total: number,
  cross: boolean,
  gap: number,
  hit: number,
  thumbMin: number
): AxisState {
  const maxScroll = total - client;
  const overflow = maxScroll > 1;
  const track = Math.max(0, client - gap * 2 - (cross ? hit : 0));
  if (!overflow || track <= 0 || total <= 0) {
    return idleAxis;
  }
  const size = Math.min(track, Math.max(thumbMin, Math.round((client / total) * track)));
  const travel = Math.max(0, track - size);
  const offset = travel === 0 ? 0 : (scroll / maxScroll) * travel;
  return { overflow: true, size, offset, travel, maxScroll };
}

function measure(el: HTMLElement, axis: ScrollAxis) {
  const gap = readLength(el, '--scroll-gap', 2);
  const hit = readLength(el, '--scroll-hit', 14);
  const thumbMin = readLength(el, '--scroll-thumb-min', 28);
  const allowX = axis !== 'vertical';
  const allowY = axis !== 'horizontal';
  const xOverflow = allowX && el.scrollWidth - el.clientWidth > 1;
  const yOverflow = allowY && el.scrollHeight - el.clientHeight > 1;

  return {
    x: allowX ? measureAxis(el.scrollLeft, el.clientWidth, el.scrollWidth, yOverflow, gap, hit, thumbMin) : idleAxis,
    y: allowY ? measureAxis(el.scrollTop, el.clientHeight, el.scrollHeight, xOverflow, gap, hit, thumbMin) : idleAxis
  };
}

function placeThumb(thumb: HTMLDivElement | null, spec: AxisState, axis: AxisName) {
  if (!thumb) {
    return;
  }
  if (axis === 'y') {
    thumb.style.height = `${spec.size}px`;
    thumb.style.transform = `translate3d(0, ${spec.offset}px, 0)`;
    return;
  }
  thumb.style.width = `${spec.size}px`;
  thumb.style.transform = `translate3d(${spec.offset}px, 0, 0)`;
}

/**
 * 悬浮滚动条。
 *
 * 原生滚动条隐藏。默认滑块盖在内容上，不占布局宽度。
 * `edge` 时滑块改到右侧留白的外缘。`gutter` 时内容自己让开滑块。
 * 悬停、滚动或拖动时出现，停下后淡出。内容没有溢出则不渲染滑块。
 */
export function Scroll({
  className,
  axis = 'both',
  edge = false,
  gutter = false,
  chain = false,
  viewportRef: viewportRefProp,
  children
}: ScrollProps) {
  const rootRef = useRef<HTMLDivElement>(null);
  const viewportRef = useRef<HTMLDivElement>(null);
  const thumbXRef = useRef<HTMLDivElement>(null);
  const thumbYRef = useRef<HTMLDivElement>(null);
  const barXRef = useRef<HTMLDivElement>(null);
  const barYRef = useRef<HTMLDivElement>(null);
  const axisRef = useRef(axis);
  const overflowRef = useRef({ x: false, y: false });
  const dragRef = useRef<DragState | null>(null);
  const frameRef = useRef(0);
  const hideXRef = useRef(0);
  const hideYRef = useRef(0);
  const lastLeftRef = useRef(0);
  const lastTopRef = useRef(0);
  const scrollingXRef = useRef(false);
  const scrollingYRef = useRef(false);
  const syncRef = useRef<() => void>(() => {});

  const [overflow, setOverflow] = useState({ x: false, y: false });
  const [hover, setHover] = useState(false);
  const [scrollingX, setScrollingX] = useState(false);
  const [scrollingY, setScrollingY] = useState(false);
  const [draggingX, setDraggingX] = useState(false);
  const [draggingY, setDraggingY] = useState(false);

  axisRef.current = axis;

  syncRef.current = () => {
    const viewport = viewportRef.current;
    if (!viewport) {
      return;
    }
    const next = measure(viewport, axisRef.current);
    placeThumb(thumbXRef.current, next.x, 'x');
    placeThumb(thumbYRef.current, next.y, 'y');
    if (next.x.overflow !== overflowRef.current.x || next.y.overflow !== overflowRef.current.y) {
      overflowRef.current = { x: next.x.overflow, y: next.y.overflow };
      setOverflow(overflowRef.current);
    }
  };

  useLayoutEffect(() => {
    const viewport = viewportRef.current;
    if (!viewport) {
      return;
    }
    const run = () => syncRef.current();
    const observer = new ResizeObserver(run);
    const watch = () => {
      observer.observe(viewport);
      for (const child of viewport.children) {
        observer.observe(child);
      }
      run();
    };
    const mutations = new MutationObserver(watch);
    watch();
    mutations.observe(viewport, { childList: true });
    return () => {
      observer.disconnect();
      mutations.disconnect();
    };
  }, []);

  useLayoutEffect(() => {
    const viewport = viewportRef.current;
    const bars = [barXRef.current, barYRef.current];
    if (!viewport) {
      return;
    }
    const onWheel = (event: WheelEvent) => {
      event.preventDefault();
      viewport.scrollLeft += event.deltaX;
      viewport.scrollTop += event.deltaY;
    };
    for (const bar of bars) {
      bar?.addEventListener('wheel', onWheel, { passive: false });
    }
    return () => {
      for (const bar of bars) {
        bar?.removeEventListener('wheel', onWheel);
      }
    };
  }, []);

  useLayoutEffect(() => {
    return () => {
      window.clearTimeout(hideXRef.current);
      window.clearTimeout(hideYRef.current);
      if (frameRef.current !== 0) {
        cancelAnimationFrame(frameRef.current);
      }
    };
  }, []);

  function markScrolling(name: AxisName) {
    const flag = name === 'y' ? scrollingYRef : scrollingXRef;
    const setFlag = name === 'y' ? setScrollingY : setScrollingX;
    const timer = name === 'y' ? hideYRef : hideXRef;
    if (!flag.current) {
      flag.current = true;
      setFlag(true);
    }
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => {
      flag.current = false;
      setFlag(false);
    }, HIDE_MS);
  }

  function onScroll() {
    if (frameRef.current !== 0) {
      return;
    }
    frameRef.current = requestAnimationFrame(() => {
      frameRef.current = 0;
      const viewport = viewportRef.current;
      if (!viewport) {
        return;
      }
      if (viewport.scrollTop !== lastTopRef.current) {
        lastTopRef.current = viewport.scrollTop;
        markScrolling('y');
      }
      if (viewport.scrollLeft !== lastLeftRef.current) {
        lastLeftRef.current = viewport.scrollLeft;
        markScrolling('x');
      }
      syncRef.current();
    });
  }

  function endDrag() {
    if (!dragRef.current) {
      return;
    }
    dragRef.current = null;
    setDraggingX(false);
    setDraggingY(false);
  }

  function onBarPointerDown(name: AxisName, event: ReactPointerEvent<HTMLDivElement>) {
    if (event.button !== 0) {
      return;
    }
    const viewport = viewportRef.current;
    if (!viewport) {
      return;
    }
    const spec = measure(viewport, axisRef.current)[name];
    if (!spec.overflow || spec.travel <= 0) {
      return;
    }
    const rect = event.currentTarget.getBoundingClientRect();
    const client = name === 'y' ? event.clientY : event.clientX;
    const origin = name === 'y' ? rect.top : rect.left;
    const local = client - origin;
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);

    let startScroll = name === 'y' ? viewport.scrollTop : viewport.scrollLeft;
    const onThumb = local >= spec.offset && local <= spec.offset + spec.size;
    if (!onThumb) {
      const ratio = (local - spec.size / 2) / spec.travel;
      startScroll = Math.min(spec.maxScroll, Math.max(0, ratio * spec.maxScroll));
      if (name === 'y') {
        viewport.scrollTop = startScroll;
      } else {
        viewport.scrollLeft = startScroll;
      }
    }

    dragRef.current = {
      axis: name,
      startClient: client,
      startScroll,
      travel: spec.travel,
      maxScroll: spec.maxScroll
    };
    if (name === 'y') {
      setDraggingY(true);
    } else {
      setDraggingX(true);
    }
  }

  function onBarPointerMove(event: ReactPointerEvent<HTMLDivElement>) {
    const drag = dragRef.current;
    const viewport = viewportRef.current;
    if (!drag || !viewport || drag.travel <= 0) {
      return;
    }
    const client = drag.axis === 'y' ? event.clientY : event.clientX;
    const next = drag.startScroll + ((client - drag.startClient) / drag.travel) * drag.maxScroll;
    const clamped = Math.min(drag.maxScroll, Math.max(0, next));
    if (drag.axis === 'y') {
      viewport.scrollTop = clamped;
    } else {
      viewport.scrollLeft = clamped;
    }
  }

  const showY = overflow.y && (hover || scrollingY || draggingY);
  const showX = overflow.x && (hover || scrollingX || draggingX);

  return (
    <div
      ref={rootRef}
      className={clsx(styles.root, className)}
      data-overflow-x={overflow.x ? 'true' : undefined}
      data-overflow-y={overflow.y ? 'true' : undefined}
      data-show-x={showX ? 'true' : undefined}
      data-show-y={showY ? 'true' : undefined}
      data-dragging-x={draggingX ? 'true' : undefined}
      data-dragging-y={draggingY ? 'true' : undefined}
      data-edge={edge ? 'true' : undefined}
      data-gutter={gutter ? 'true' : undefined}
      data-chain={chain ? 'true' : undefined}
      onPointerEnter={() => setHover(true)}
      onPointerLeave={() => setHover(false)}
    >
      <div
        ref={(node) => {
          viewportRef.current = node;
          setRef(viewportRefProp, node);
        }}
        className={styles.viewport}
        data-axis={axis}
        onScroll={onScroll}
      >
        {children}
      </div>
      <div
        ref={barYRef}
        className={styles.barY}
        hidden={!overflow.y}
        aria-hidden="true"
        onPointerDown={(event) => onBarPointerDown('y', event)}
        onPointerMove={onBarPointerMove}
        onPointerUp={endDrag}
        onPointerCancel={endDrag}
      >
        <div ref={thumbYRef} className={styles.thumbY} />
      </div>
      <div
        ref={barXRef}
        className={styles.barX}
        hidden={!overflow.x}
        aria-hidden="true"
        onPointerDown={(event) => onBarPointerDown('x', event)}
        onPointerMove={onBarPointerMove}
        onPointerUp={endDrag}
        onPointerCancel={endDrag}
      >
        <div ref={thumbXRef} className={styles.thumbX} />
      </div>
    </div>
  );
}
