import { InfoIcon } from '@phosphor-icons/react';
import clsx from 'clsx';
import type { FocusEvent, PointerEvent, ReactElement, ReactNode, Ref } from 'react';
import { cloneElement, isValidElement, useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import styles from './index.module.css';

type Placement = 'top' | 'bottom';

type TooltipProps = {
  /** 悬停、聚焦或长按时出现的说明 */
  title: ReactNode;
  /** 必须是能接收 ref 和指针事件的单个元素 */
  children: ReactElement;
  /** 优先出现在锚点上方，放不下时翻到下方 */
  placement?: Placement;
  /** 加在说明气泡上 */
  className?: string;
};

type TriggerProps = {
  ref?: Ref<HTMLElement>;
  onPointerEnter?: (event: PointerEvent<HTMLElement>) => void;
  onPointerLeave?: (event: PointerEvent<HTMLElement>) => void;
  onPointerDown?: (event: PointerEvent<HTMLElement>) => void;
  onPointerUp?: (event: PointerEvent<HTMLElement>) => void;
  onPointerCancel?: (event: PointerEvent<HTMLElement>) => void;
  onFocus?: (event: FocusEvent<HTMLElement>) => void;
  onBlur?: (event: FocusEvent<HTMLElement>) => void;
  'aria-describedby'?: string;
};

const enterDelay = 100;
const leaveDelay = 200;
const touchDelay = 700;
const touchLeave = 1500;

function assignRef<T>(ref: Ref<T> | undefined, value: T | null) {
  if (typeof ref === 'function') {
    ref(value);
    return;
  }
  if (ref) {
    ref.current = value;
  }
}

function call<E>(handler: ((event: E) => void) | undefined, event: E) {
  handler?.(event);
}

function place(anchor: DOMRect, width: number, height: number, placement: Placement) {
  const gap = 8;
  const margin = 8;
  let used = placement;
  let top = used === 'top' ? anchor.top - height - gap : anchor.bottom + gap;
  if (used === 'top' && top < margin) {
    used = 'bottom';
    top = anchor.bottom + gap;
  } else if (used === 'bottom' && top + height > window.innerHeight - margin) {
    used = 'top';
    top = anchor.top - height - gap;
  }
  const left = Math.min(
    Math.max(margin, anchor.left + anchor.width / 2 - width / 2),
    window.innerWidth - width - margin
  );
  const arrow = Math.min(Math.max(12, anchor.left + anchor.width / 2 - left), width - 12);
  return { top, left, placement: used, arrow };
}

/** 锚点旁的说明。悬停、聚焦或长按出现，移开后收回。行为参考 Material UI Tooltip。 */
export function Tooltip({ title, children, placement = 'top', className }: TooltipProps) {
  const id = useId();
  const anchorRef = useRef<HTMLElement>(null);
  const bubbleRef = useRef<HTMLDivElement>(null);
  const enterTimer = useRef(0);
  const leaveTimer = useRef(0);
  const fromTouch = useRef(false);
  const openRef = useRef(false);
  const [open, setOpen] = useState(false);
  const [shown, setShown] = useState(false);
  const [spot, setSpot] = useState({ top: 0, left: 0, placement, arrow: 16 });
  openRef.current = open;

  function show(delay: number) {
    window.clearTimeout(enterTimer.current);
    window.clearTimeout(leaveTimer.current);
    if (delay <= 0) {
      setOpen(true);
      return;
    }
    enterTimer.current = window.setTimeout(() => setOpen(true), delay);
  }

  function hide(delay: number) {
    window.clearTimeout(enterTimer.current);
    if (delay <= 0) {
      window.clearTimeout(leaveTimer.current);
      setOpen(false);
      return;
    }
    leaveTimer.current = window.setTimeout(() => setOpen(false), delay);
  }

  useLayoutEffect(() => {
    if (!open) {
      return;
    }
    const anchor = anchorRef.current;
    const bubble = bubbleRef.current;
    if (!anchor || !bubble) {
      return;
    }
    function update() {
      const currentAnchor = anchorRef.current;
      const currentBubble = bubbleRef.current;
      if (!currentAnchor || !currentBubble) {
        return;
      }
      // offsetWidth 不受 scale(0) 影响。getBoundingClientRect 会把还没展开的气泡量成 0。
      setSpot(
        place(currentAnchor.getBoundingClientRect(), currentBubble.offsetWidth, currentBubble.offsetHeight, placement)
      );
    }
    update();
    const frame = requestAnimationFrame(() => setShown(true));
    window.addEventListener('resize', update);
    window.addEventListener('scroll', update, true);
    return () => {
      cancelAnimationFrame(frame);
      window.removeEventListener('resize', update);
      window.removeEventListener('scroll', update, true);
    };
  }, [open, placement]);

  useEffect(() => {
    if (open) {
      return;
    }
    setShown(false);
  }, [open]);

  useEffect(() => {
    if (!open) {
      return;
    }
    function onKey(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        setOpen(false);
      }
    }
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, [open]);

  useEffect(() => {
    return () => {
      window.clearTimeout(enterTimer.current);
      window.clearTimeout(leaveTimer.current);
    };
  }, []);

  if (!isValidElement<TriggerProps>(children)) {
    return children;
  }

  const trigger = children;

  function pointerEnter(event: PointerEvent<HTMLElement>) {
    call(trigger.props.onPointerEnter, event);
    if (event.pointerType === 'touch') {
      return;
    }
    show(enterDelay);
  }

  function pointerLeave(event: PointerEvent<HTMLElement>) {
    call(trigger.props.onPointerLeave, event);
    if (event.pointerType === 'touch') {
      return;
    }
    hide(leaveDelay);
  }

  function pointerDown(event: PointerEvent<HTMLElement>) {
    call(trigger.props.onPointerDown, event);
    if (event.pointerType !== 'touch') {
      return;
    }
    fromTouch.current = true;
    show(touchDelay);
  }

  function pointerUp(event: PointerEvent<HTMLElement>) {
    call(trigger.props.onPointerUp, event);
    if (event.pointerType !== 'touch') {
      return;
    }
    window.clearTimeout(enterTimer.current);
    if (openRef.current) {
      hide(touchLeave);
    }
  }

  function focus(event: FocusEvent<HTMLElement>) {
    call(trigger.props.onFocus, event);
    if (fromTouch.current) {
      return;
    }
    show(0);
  }

  function blur(event: FocusEvent<HTMLElement>) {
    call(trigger.props.onBlur, event);
    fromTouch.current = false;
    const next = event.relatedTarget;
    if (next instanceof Node && bubbleRef.current?.contains(next)) {
      return;
    }
    hide(0);
  }

  const describedBy = [trigger.props['aria-describedby'], open ? id : undefined].filter(Boolean).join(' ') || undefined;

  return (
    <>
      {cloneElement(trigger, {
        ref: (node: HTMLElement | null) => {
          anchorRef.current = node;
          assignRef(trigger.props.ref, node);
        },
        onPointerEnter: pointerEnter,
        onPointerLeave: pointerLeave,
        onPointerDown: pointerDown,
        onPointerUp: pointerUp,
        onPointerCancel: (event: PointerEvent<HTMLElement>) => {
          call(trigger.props.onPointerCancel, event);
          fromTouch.current = false;
          hide(0);
        },
        onFocus: focus,
        onBlur: blur,
        'aria-describedby': describedBy
      })}
      {open
        ? createPortal(
            <div
              ref={bubbleRef}
              id={id}
              role="tooltip"
              className={clsx(styles.bubble, className)}
              data-open={shown ? 'true' : undefined}
              data-placement={spot.placement}
              style={{
                top: spot.top,
                left: spot.left,
                transformOrigin: `${spot.arrow}px ${spot.placement === 'top' ? 'bottom' : 'top'}`
              }}
              onPointerEnter={() => show(0)}
              onPointerLeave={() => hide(leaveDelay)}
            >
              {title}
              <span className={styles.arrow} style={{ left: spot.arrow }} />
            </div>,
            document.body
          )
        : null}
    </>
  );
}

type InfoTipProps = {
  /** 按钮的无障碍名称，例如「距离说明」 */
  label: string;
  /** 浮层里的详细说明 */
  title: string;
  /** 加在说明按钮上 */
  className?: string;
};

/** 参数名后面的说明按钮。悬停或聚焦后展开详细介绍。 */
export function InfoTip({ label, title, className }: InfoTipProps) {
  return (
    <Tooltip title={title}>
      <button type="button" className={clsx(styles.trigger, className)} aria-label={label}>
        <InfoIcon className={styles.icon} weight="regular" />
      </button>
    </Tooltip>
  );
}
