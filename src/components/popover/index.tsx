import clsx from 'clsx';
import type { MouseEvent, ReactNode } from 'react';
import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import styles from './index.module.css';

type VerticalOrigin = 'top' | 'center' | 'bottom';
type HorizontalOrigin = 'left' | 'center' | 'right';

/** 锚点或浮层上的对齐位置。数字是相对该边的像素偏移，和 Material UI 一致。 */
export type PopoverOrigin = {
  vertical: VerticalOrigin | number;
  horizontal: HorizontalOrigin | number;
};

/** 没有真实节点时，用 getBoundingClientRect 描述锚点。 */
export type PopoverVirtualElement = {
  nodeType: 1;
  getBoundingClientRect: () => DOMRect;
};

export type PopoverAnchor = HTMLElement | PopoverVirtualElement;

type PopoverProps = {
  open: boolean;
  anchorEl: PopoverAnchor | null;
  onClose: () => void;
  children: ReactNode;
  /** 加在浮层纸面上，用来补宽度和内边距 */
  className?: string;
  anchorOrigin?: PopoverOrigin;
  transformOrigin?: PopoverOrigin;
  /** 距离视口边缘至少留出的间距，默认 16 */
  marginThreshold?: number;
  /** 沿锚点外侧留出的间距，单位 px */
  gap?: number;
  label?: string;
};

const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

const layers: Array<() => void> = [];

function axisOffset(side: number | string, size: number, center: string, end: string) {
  if (typeof side === 'number') {
    return side;
  }
  if (side === center) {
    return size / 2;
  }
  if (side === end) {
    return size;
  }
  return 0;
}

function originOffset(origin: PopoverOrigin, size: { width: number; height: number }) {
  return {
    horizontal: axisOffset(origin.horizontal, size.width, 'center', 'right'),
    vertical: axisOffset(origin.vertical, size.height, 'center', 'bottom')
  };
}

function withGap(top: number, left: number, anchor: PopoverOrigin, transform: PopoverOrigin, gap: number) {
  let nextTop = top;
  let nextLeft = left;
  if (anchor.vertical === 'bottom' && transform.vertical === 'top') {
    nextTop += gap;
  } else if (anchor.vertical === 'top' && transform.vertical === 'bottom') {
    nextTop -= gap;
  }
  if (anchor.horizontal === 'right' && transform.horizontal === 'left') {
    nextLeft += gap;
  } else if (anchor.horizontal === 'left' && transform.horizontal === 'right') {
    nextLeft -= gap;
  }
  return { left: nextLeft, top: nextTop };
}

function placePopover(
  anchorRect: DOMRect,
  paper: { width: number; height: number },
  anchorOrigin: PopoverOrigin,
  transformOrigin: PopoverOrigin,
  marginThreshold: number,
  gap: number
) {
  const anchorOffset = originOffset(anchorOrigin, anchorRect);
  const paperOffset = originOffset(transformOrigin, paper);
  const gapped = withGap(
    anchorRect.top + anchorOffset.vertical - paperOffset.vertical,
    anchorRect.left + anchorOffset.horizontal - paperOffset.horizontal,
    anchorOrigin,
    transformOrigin,
    gap
  );
  let top = gapped.top;
  let left = gapped.left;
  let originX = paperOffset.horizontal;
  let originY = paperOffset.vertical;
  const heightLimit = window.innerHeight - marginThreshold;
  const widthLimit = window.innerWidth - marginThreshold;

  if (top < marginThreshold) {
    const diff = top - marginThreshold;
    top -= diff;
    originY += diff;
  } else if (top + paper.height > heightLimit) {
    const diff = top + paper.height - heightLimit;
    top -= diff;
    originY += diff;
  }

  if (left < marginThreshold) {
    const diff = left - marginThreshold;
    left -= diff;
    originX += diff;
  } else if (left + paper.width > widthLimit) {
    const diff = left + paper.width - widthLimit;
    left -= diff;
    originX += diff;
  }

  return {
    left,
    top,
    transformOrigin: `${originX}px ${originY}px`
  };
}

function motionMs(paper: HTMLElement) {
  const motion = window.getComputedStyle(paper);
  const frozen = motion.transitionDuration.split(',').every((part) => Number.parseFloat(part) === 0);
  if (frozen) {
    return 0;
  }
  const value = Number.parseFloat(motion.getPropertyValue('--popover-motion'));
  return Number.isFinite(value) ? value : 195;
}

/** 锚在元素旁的浮层。Portal 到 body，避免被对话框裁切；点外面或按 Esc 关闭。 */
export function Popover({
  open,
  anchorEl,
  onClose,
  children,
  className,
  anchorOrigin = { horizontal: 'left', vertical: 'top' },
  transformOrigin = { horizontal: 'left', vertical: 'top' },
  marginThreshold = 16,
  gap = 0,
  label
}: PopoverProps) {
  const [mounted, setMounted] = useState(open && anchorEl != null);
  const [entered, setEntered] = useState(false);
  const paperRef = useRef<HTMLDivElement>(null);
  const onCloseRef = useRef(onClose);
  const optionsRef = useRef({ anchorOrigin, gap, marginThreshold, transformOrigin });
  onCloseRef.current = onClose;
  optionsRef.current = { anchorOrigin, gap, marginThreshold, transformOrigin };

  if (open && anchorEl != null && !mounted) {
    setMounted(true);
  }

  useLayoutEffect(() => {
    if (!mounted || anchorEl == null) {
      return;
    }

    function update() {
      const paper = paperRef.current;
      if (!paper || anchorEl == null) {
        return;
      }
      if (anchorEl instanceof HTMLElement && !anchorEl.isConnected) {
        return;
      }
      const options = optionsRef.current;
      const next = placePopover(
        anchorEl.getBoundingClientRect(),
        { height: paper.offsetHeight, width: paper.offsetWidth },
        options.anchorOrigin,
        options.transformOrigin,
        options.marginThreshold,
        options.gap
      );
      paper.style.top = `${next.top}px`;
      paper.style.left = `${next.left}px`;
      paper.style.transformOrigin = next.transformOrigin;
    }

    update();
    const paper = paperRef.current;
    const observer = new ResizeObserver(update);
    if (paper) {
      observer.observe(paper);
    }
    window.addEventListener('resize', update);
    window.addEventListener('scroll', update, true);
    return () => {
      observer.disconnect();
      window.removeEventListener('resize', update);
      window.removeEventListener('scroll', update, true);
    };
  }, [anchorEl, mounted]);

  useEffect(() => {
    if (!mounted) {
      return;
    }
    if (open) {
      const frame = requestAnimationFrame(() => setEntered(true));
      return () => cancelAnimationFrame(frame);
    }
    const ms = paperRef.current ? motionMs(paperRef.current) : 0;
    const timer = window.setTimeout(() => {
      setEntered(false);
      setMounted(false);
    }, ms);
    return () => window.clearTimeout(timer);
  }, [mounted, open]);

  useEffect(() => {
    if (!open || !mounted) {
      return;
    }
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const frame = requestAnimationFrame(() => paperRef.current?.focus({ preventScroll: true }));
    return () => {
      cancelAnimationFrame(frame);
      if (previous?.isConnected) {
        previous.focus({ preventScroll: true });
      }
    };
  }, [mounted, open]);

  useEffect(() => {
    if (!open) {
      return;
    }

    const close = () => onCloseRef.current();
    layers.push(close);

    function onKey(event: KeyboardEvent) {
      if (layers[layers.length - 1] !== close) {
        return;
      }
      if (event.key === 'Escape') {
        event.preventDefault();
        event.stopPropagation();
        const index = layers.indexOf(close);
        if (index >= 0) {
          layers.splice(index, 1);
        }
        close();
        return;
      }
      if (event.key !== 'Tab') {
        return;
      }
      const paper = paperRef.current;
      if (!paper) {
        return;
      }
      const nodes = [...paper.querySelectorAll<HTMLElement>(FOCUSABLE)];
      if (nodes.length === 0) {
        event.preventDefault();
        return;
      }
      const first = nodes[0];
      const last = nodes[nodes.length - 1];
      const active = document.activeElement;
      const inside = active instanceof Node && paper.contains(active);
      if (event.shiftKey && (active === first || !inside)) {
        event.preventDefault();
        last.focus();
        return;
      }
      if (!event.shiftKey && (active === last || !inside)) {
        event.preventDefault();
        first.focus();
      }
    }

    document.addEventListener('keydown', onKey, true);
    return () => {
      document.removeEventListener('keydown', onKey, true);
      const index = layers.indexOf(close);
      if (index >= 0) {
        layers.splice(index, 1);
      }
    };
  }, [open]);

  function onBackdropMouseDown(event: MouseEvent<HTMLDivElement>) {
    event.preventDefault();
  }

  if (!mounted) {
    return null;
  }

  return createPortal(
    <div className={styles.root} data-open={open && entered ? 'true' : undefined}>
      <div className={styles.backdrop} aria-hidden="true" onMouseDown={onBackdropMouseDown} onClick={onClose} />
      <div
        ref={paperRef}
        className={clsx(styles.paper, className)}
        role="dialog"
        aria-modal="true"
        aria-label={label}
        tabIndex={-1}
      >
        {children}
      </div>
    </div>,
    document.body
  );
}
