import clsx from 'clsx';
import type { CSSProperties, PointerEvent as ReactPointerEvent } from 'react';
import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import type { Plan } from '@/bridge/plan';
import { rippleHost, useRipple } from '@/components/ripple';
import { Scroll } from '@/components/scroll';
import { startOfLocalDay } from '@/utils/day';
import styles from '../calendar.module.css';
import { fullWeekday, hasTimeRange, isTimedSingle, layoutBars, weekdayLabel } from '../layout';
import { PlanChip } from './plan-chip';

type TimeGridProps = {
  days: number[];
  today: number;
  plans: Plan[];
  hueOf: (plan: Plan) => number;
  onOpenDay: (day: number) => void;
  onCreateAllDay: (day: number) => void;
  onCreateRange: (day: number, fromMinute: number, toMinute: number) => void;
  onOpen: (plan: Plan) => void;
  onToggle: (plan: Plan) => void;
  /** 拖动单日任务。end 为空表示没有结束时间 */
  onMove: (plan: Plan, day: number, startMinute: number, endMinute: number | null) => void;
};

type Box = {
  left: number;
  top: number;
  width: number;
  height: number;
};

type SpanBox = Box & {
  plan: Plan;
};

type CreateDrag = Box & {
  kind: 'create';
  date: number;
  startSlot: number;
  slot: number;
  x: number;
  y: number;
  moved: boolean;
};

type TaskDrag = Box & {
  kind: 'task';
  mode: 'move' | 'start' | 'end';
  planId: string;
  originDate: number;
  originStart: number;
  originEnd: number | null;
  grab: number;
  date: number;
  start: number;
  end: number | null;
  x: number;
  y: number;
  moved: boolean;
};

type Drag = CreateDrag | TaskDrag;

type NowLine = {
  top: number;
  left: number;
  width: number;
  label: string;
};

const OPEN_START = 8 * 60;
const OPEN_END = 22 * 60;

/** 周和日共用的时间轴。一格一小时，拖动按半小时吸附 */
export function TimeGrid({
  days,
  today,
  plans,
  hueOf,
  onOpenDay,
  onCreateAllDay,
  onCreateRange,
  onOpen,
  onToggle,
  onMove
}: TimeGridProps) {
  const cols = days.length;
  const bodyRef = useRef<HTMLDivElement>(null);
  const timedRef = useRef<Plan[]>([]);
  const dragRef = useRef<Drag | null>(null);
  const skipOpen = useRef(false);
  const [drag, setDrag] = useState<Drag | null>(null);
  const [spans, setSpans] = useState<SpanBox[]>([]);
  const [nowLine, setNowLine] = useState<NowLine | null>(null);
  const [clock, setClock] = useState(() => Date.now());
  const daysRef = useRef(days);
  daysRef.current = days;
  const timed = plans.filter(isTimedSingle);
  const bars = layoutBars(days, plans);
  const laneCount = bars.reduce((max, segment) => Math.max(max, segment.lane + 1), 0);
  const hours = visibleHours(timed);
  const spanKey = timed.map((plan) => `${plan.id}:${plan.scheduledAt}:${plan.dueAt}:${plan.status}`).join('|');
  const dayKey = days.join(',');
  timedRef.current = timed;
  const draggingId = drag?.kind === 'task' && drag.moved ? drag.planId : null;

  useEffect(() => {
    const timer = window.setInterval(() => setClock(Date.now()), 15_000);
    return () => window.clearInterval(timer);
  }, []);

  useLayoutEffect(() => {
    const root = bodyRef.current;
    if (!root) {
      return;
    }
    root.dataset.measured = `${spanKey}|${dayKey}|${clock}|${today}`;
    setSpans(measureSpans(root, timedRef.current));
    setNowLine(measureNow(root, today, daysRef.current, clock));
  }, [spanKey, dayKey, clock, today]);

  function openIfClick(plan: Plan) {
    if (skipOpen.current) {
      skipOpen.current = false;
      return;
    }
    onOpen(plan);
  }

  function pointerDown(event: ReactPointerEvent<HTMLDivElement>) {
    if (event.button !== 0) {
      return;
    }
    const target = event.target;
    if (!(target instanceof Element) || target.closest('[data-check]')) {
      return;
    }
    const move = target.closest<HTMLElement>('[data-move]');
    if (move) {
      const plan = timedRef.current.find((item) => item.id === move.dataset.move);
      if (!plan || !isTimedSingle(plan) || plan.scheduledAt == null) {
        return;
      }
      const edge = target.closest<HTMLElement>('[data-edge]')?.dataset.edge;
      const mode = edge === 'start' || edge === 'end' ? edge : 'move';
      if (mode !== 'move' && !hasTimeRange(plan)) {
        return;
      }
      const root = bodyRef.current;
      const hit = hitAt(event.clientX, event.clientY);
      if (!root) {
        return;
      }
      const originDate = startOfLocalDay(plan.scheduledAt);
      const originStart = floorHalf(plan.scheduledAt);
      const originEnd = hasTimeRange(plan) && plan.dueAt != null ? ceilHalf(plan.dueAt) : null;
      const box = boxFor(root, originDate, originStart, originEnd) ?? emptyBox();
      const state: TaskDrag = {
        kind: 'task',
        mode,
        planId: plan.id,
        originDate,
        originStart,
        originEnd,
        grab: hit?.minute ?? originStart,
        date: originDate,
        start: originStart,
        end: originEnd,
        x: event.clientX,
        y: event.clientY,
        moved: false,
        ...box
      };
      dragRef.current = state;
      setDrag(state);
      event.currentTarget.setPointerCapture(event.pointerId);
      return;
    }
    if (target.closest('[data-chip]')) {
      return;
    }
    const hit = hitAt(event.clientX, event.clientY);
    const root = event.currentTarget;
    if (!hit) {
      return;
    }
    event.preventDefault();
    root.setPointerCapture(event.pointerId);
    const box = boxFor(root, hit.date, hit.minute, hit.minute + 30) ?? emptyBox();
    const state: CreateDrag = {
      kind: 'create',
      date: hit.date,
      startSlot: hit.minute,
      slot: hit.minute,
      x: event.clientX,
      y: event.clientY,
      moved: false,
      ...box
    };
    dragRef.current = state;
    setDrag(state);
  }

  function pointerMove(event: ReactPointerEvent<HTMLDivElement>) {
    const current = dragRef.current;
    const root = bodyRef.current;
    if (!current || !root) {
      return;
    }
    const hit = hitAt(event.clientX, event.clientY);
    if (current.kind === 'create') {
      const slot = hit && hit.date === current.date ? hit.minute : current.slot;
      const moved =
        current.moved ||
        Math.hypot(event.clientX - current.x, event.clientY - current.y) > 6 ||
        slot !== current.startSlot;
      if (slot === current.slot && moved === current.moved) {
        return;
      }
      const from = Math.min(current.startSlot, slot);
      const to = Math.max(current.startSlot, slot);
      const box = boxFor(root, current.date, from, to + 30) ?? emptyBox();
      const next: CreateDrag = { ...current, ...box, slot, moved };
      dragRef.current = next;
      setDrag(next);
      return;
    }
    if (!hit) {
      return;
    }
    const bounds = gridBounds(root);
    let date = current.mode === 'move' ? hit.date : current.originDate;
    let start = current.originStart;
    let end = current.originEnd;
    if (current.mode === 'move') {
      const duration = end == null ? 0 : end - start;
      start = clampStart(start + hit.minute - current.grab, duration, bounds);
      end = end == null ? null : start + duration;
    } else if (current.mode === 'start' && end != null) {
      start = Math.min(Math.max(hit.minute, bounds.start), end - 30);
    } else if (current.mode === 'end' && end != null) {
      end = Math.max(Math.min(hit.minute + 30, bounds.end), start + 30);
    }
    if (!days.includes(date)) {
      date = current.date;
    }
    const moved =
      current.moved ||
      Math.hypot(event.clientX - current.x, event.clientY - current.y) > 6 ||
      start !== current.originStart ||
      end !== current.originEnd ||
      date !== current.originDate;
    if (!moved && date === current.date && start === current.start && end === current.end) {
      return;
    }
    const box = boxFor(root, date, start, end) ?? emptyBox();
    const next: TaskDrag = { ...current, ...box, date, start, end, moved };
    dragRef.current = next;
    setDrag(next);
  }

  function pointerUp() {
    const current = dragRef.current;
    dragRef.current = null;
    setDrag(null);
    if (!current) {
      return;
    }
    if (current.kind === 'task') {
      skipOpen.current = true;
      holdClick(() => {
        skipOpen.current = false;
      });
    }
    if (!current.moved) {
      if (current.kind === 'task') {
        const plan = timedRef.current.find((item) => item.id === current.planId);
        if (plan) {
          onOpen(plan);
        }
      }
      return;
    }
    if (current.kind === 'create') {
      const from = Math.min(current.startSlot, current.slot);
      const to = Math.max(current.startSlot, current.slot) + 30;
      onCreateRange(current.date, from, to);
      return;
    }
    const plan = timedRef.current.find((item) => item.id === current.planId);
    if (!plan) {
      return;
    }
    const same =
      current.date === current.originDate && current.start === current.originStart && current.end === current.originEnd;
    if (same) {
      return;
    }
    onMove(plan, current.date, current.start, current.end);
  }

  function pointerCancel() {
    dragRef.current = null;
    setDrag(null);
  }

  const ghost = drag?.kind === 'task' && drag.moved ? drag : null;
  const ghostPlan = ghost ? timed.find((item) => item.id === ghost.planId) : undefined;
  const dragging =
    drag?.kind === 'create' && drag.moved
      ? 'resize'
      : drag?.kind === 'task' && drag.moved
        ? drag.mode === 'move'
          ? 'move'
          : 'resize'
        : undefined;

  return (
    <div className={styles.schedule} style={{ '--cols': cols } as CSSProperties}>
      {cols > 1 ? (
        <div className={styles.heads}>
          <div className={styles.gutter} />
          {days.map((day) => (
            <div key={day} className={styles.colHead}>
              <span className={styles.wd}>{weekdayLabel(day)}</span>
              <DayJump today={startOfLocalDay(day) === today} onClick={() => onOpenDay(day)}>
                {new Date(day).getDate()}
              </DayJump>
            </div>
          ))}
        </div>
      ) : (
        <div className={styles.dayBanner}>
          <span className={styles.wd}>{fullWeekday(days[0] ?? today)}</span>
          <span className={styles.colNum} data-today={startOfLocalDay(days[0] ?? today) === today ? 'true' : undefined}>
            {new Date(days[0] ?? today).getDate()}
          </span>
        </div>
      )}
      <div className={styles.allday}>
        <div className={styles.gutter}>全天</div>
        <div className={styles.alldayCols} style={{ minHeight: Math.max(36, 8 + laneCount * 26) }}>
          <div className={styles.alldayHits}>
            {days.map((day) => (
              <button
                key={day}
                type="button"
                className={styles.alldayHit}
                aria-label={`在${new Date(day).getMonth() + 1}月${new Date(day).getDate()}日添加全天计划`}
                onClick={() => onCreateAllDay(day)}
              />
            ))}
          </div>
          {bars.map((segment) => (
            <PlanChip
              key={`${segment.plan.id}-${segment.col}`}
              plan={segment.plan}
              hue={hueOf(segment.plan)}
              className={styles.bar}
              before={segment.before}
              after={segment.after}
              style={barStyle(segment.col, segment.span, segment.lane, cols, segment.before, segment.after)}
              onOpen={onOpen}
              onToggle={onToggle}
            />
          ))}
        </div>
      </div>
      <Scroll className={styles.timeScroll} axis="vertical">
        <div
          ref={bodyRef}
          className={styles.timeBody}
          data-dragging={dragging}
          onPointerDown={pointerDown}
          onPointerMove={pointerMove}
          onPointerUp={pointerUp}
          onPointerCancel={pointerCancel}
        >
          {hours.map((hour) => (
            <div key={hour} className={styles.trow} data-hour={hour}>
              <div className={styles.gutter}>
                <span>{hourLabel(hour)}</span>
              </div>
              {days.map((day) => (
                <div key={day} className={styles.tcell} data-date={day}>
                  {timed
                    .filter((plan) => sameDay(plan, day) && hourOf(plan) === hour && !hasTimeRange(plan))
                    .map((plan) => (
                      <div
                        key={plan.id}
                        className={styles.pointHost}
                        data-move={plan.id}
                        style={{ visibility: draggingId === plan.id ? 'hidden' : undefined }}
                      >
                        <PlanChip
                          plan={plan}
                          hue={hueOf(plan)}
                          className={styles.block}
                          onOpen={openIfClick}
                          onToggle={onToggle}
                        />
                      </div>
                    ))}
                </div>
              ))}
            </div>
          ))}
          {spans.map((box) => (
            <div
              key={box.plan.id}
              className={styles.spanHost}
              data-move={box.plan.id}
              style={{
                top: box.top,
                left: box.left,
                width: box.width,
                height: box.height,
                visibility: draggingId === box.plan.id ? 'hidden' : undefined
              }}
            >
              {hasTimeRange(box.plan) ? <span className={styles.edge} data-edge="start" /> : null}
              <PlanChip plan={box.plan} hue={hueOf(box.plan)} onOpen={openIfClick} onToggle={onToggle} />
              {hasTimeRange(box.plan) ? <span className={styles.edge} data-edge="end" /> : null}
            </div>
          ))}
          {drag?.kind === 'create' ? (
            <div
              className={styles.dragSel}
              style={{ top: drag.top, left: drag.left, width: drag.width, height: drag.height }}
            >
              {rangeLabel(Math.min(drag.startSlot, drag.slot), Math.max(drag.startSlot, drag.slot) + 30)}
            </div>
          ) : null}
          {ghost && ghostPlan ? (
            <div
              className={styles.dragGhost}
              style={
                {
                  top: ghost.top,
                  left: ghost.left,
                  width: ghost.width,
                  height: ghost.height,
                  '--hue': hueOf(ghostPlan)
                } as CSSProperties
              }
            >
              <span className={styles.ghostTitle}>{ghostPlan.title}</span>
              <span>{rangeLabel(ghost.start, ghost.end)}</span>
            </div>
          ) : null}
          {nowLine ? (
            <div className={styles.nowLine} style={{ top: nowLine.top, left: nowLine.left, width: nowLine.width }}>
              <span className={styles.nowLabel}>{nowLine.label}</span>
            </div>
          ) : null}
        </div>
      </Scroll>
    </div>
  );
}

/** 拖动结束后吞掉紧接着的 click，避免松开时把详情抽屉打开 */
function holdClick(done: () => void) {
  const stop = (event: Event) => {
    event.preventDefault();
    event.stopPropagation();
    window.removeEventListener('click', stop, true);
  };
  window.addEventListener('click', stop, true);
  window.setTimeout(() => {
    window.removeEventListener('click', stop, true);
    done();
  }, 0);
}

function DayJump({ today, children, onClick }: { today: boolean; children: number; onClick: () => void }) {
  const ripple = useRipple();

  return (
    <button
      type="button"
      className={clsx(rippleHost, styles.colNum)}
      data-today={today ? 'true' : undefined}
      aria-current={today ? 'date' : undefined}
      onClick={onClick}
      onPointerDown={ripple.onPointerDown}
      onKeyDown={ripple.onKeyDown}
    >
      {ripple.nodes}
      {children}
    </button>
  );
}

function sameDay(plan: Plan, day: number): boolean {
  return plan.scheduledAt != null && startOfLocalDay(plan.scheduledAt) === startOfLocalDay(day);
}

function minutesOf(time: number): number {
  const date = new Date(time);
  return date.getHours() * 60 + date.getMinutes();
}

function hourOf(plan: Plan): number {
  return Math.floor(minutesOf(plan.scheduledAt ?? 0) / 60) * 60;
}

function floorHalf(time: number): number {
  return Math.floor(minutesOf(time) / 30) * 30;
}

function ceilHalf(time: number): number {
  const minutes = minutesOf(time);
  const floored = Math.floor(minutes / 30) * 30;
  return minutes === floored ? floored : floored + 30;
}

/** 8:00 到 22:00，一格一小时。更早或更晚的计划会把格子撑出去 */
function visibleHours(plans: Plan[]): number[] {
  let start = OPEN_START;
  let end = OPEN_END;
  for (const plan of plans) {
    if (!isTimedSingle(plan) || plan.scheduledAt == null) {
      continue;
    }
    const from = Math.floor(minutesOf(plan.scheduledAt) / 60) * 60;
    const finish = plan.dueAt != null && plan.dueAt > plan.scheduledAt ? plan.dueAt : plan.scheduledAt;
    const to = Math.ceil(minutesOf(finish) / 60) * 60;
    start = Math.min(start, from);
    end = Math.max(end, Math.max(to, from + 60));
  }
  const hours: number[] = [];
  for (let cursor = start; cursor < end; cursor += 60) {
    hours.push(cursor);
  }
  return hours;
}

function hourLabel(minutes: number): string {
  if (minutes === 12 * 60) {
    return '正午';
  }
  return `${String(Math.floor(minutes / 60)).padStart(2, '0')}:00`;
}

function rangeLabel(start: number, end: number | null): string {
  const left = `${Math.floor(start / 60)}:${String(start % 60).padStart(2, '0')}`;
  if (end == null) {
    return left;
  }
  return `${left}–${Math.floor(end / 60)}:${String(end % 60).padStart(2, '0')}`;
}

function barStyle(
  col: number,
  span: number,
  lane: number,
  cols: number,
  before: boolean,
  after: boolean
): CSSProperties {
  const leftInset = before ? 0 : 4;
  const rightInset = after ? 0 : 4;
  return {
    top: 4 + lane * 26,
    left: `calc(${(col / cols) * 100}% + ${leftInset}px)`,
    width: `calc(${(span / cols) * 100}% - ${leftInset + rightInset}px)`
  };
}

function emptyBox(): Box {
  return { left: 0, top: 0, width: 0, height: 0 };
}

function hitAt(clientX: number, clientY: number): { date: number; minute: number } | null {
  for (const node of document.elementsFromPoint(clientX, clientY)) {
    if (!(node instanceof Element)) {
      continue;
    }
    const cell = node.closest<HTMLElement>('[data-date]');
    const row = cell?.closest<HTMLElement>('[data-hour]');
    if (!cell || !row) {
      continue;
    }
    const hour = Number(row.dataset.hour);
    const rect = row.getBoundingClientRect();
    const ratio = rect.height === 0 ? 0 : (clientY - rect.top) / rect.height;
    return { date: Number(cell.dataset.date), minute: hour + (ratio >= 0.5 ? 30 : 0) };
  }
  return null;
}

function gridBounds(root: HTMLElement): { start: number; end: number } {
  const rows = [...root.querySelectorAll<HTMLElement>('[data-hour]')];
  const first = Number(rows[0]?.dataset.hour);
  const last = Number(rows[rows.length - 1]?.dataset.hour);
  if (!Number.isFinite(first) || !Number.isFinite(last)) {
    return { start: OPEN_START, end: OPEN_END };
  }
  return { start: first, end: last + 60 };
}

function clampStart(start: number, duration: number, bounds: { start: number; end: number }): number {
  const length = duration === 0 ? 30 : duration;
  let next = start;
  if (next < bounds.start) {
    next = bounds.start;
  }
  if (next + length > bounds.end) {
    next = Math.max(bounds.start, bounds.end - length);
  }
  return next;
}

function minuteY(root: HTMLElement, minute: number): number | null {
  const hour = Math.floor(minute / 60) * 60;
  const row = root.querySelector<HTMLElement>(`[data-hour="${hour}"]`);
  const rootRect = root.getBoundingClientRect();
  if (row) {
    const rect = row.getBoundingClientRect();
    return rect.top - rootRect.top + ((minute - hour) / 60) * rect.height;
  }
  if (minute % 60 === 0) {
    const previous = root.querySelector<HTMLElement>(`[data-hour="${hour - 60}"]`);
    if (!previous) {
      return null;
    }
    return previous.getBoundingClientRect().bottom - rootRect.top;
  }
  return null;
}

function boxFor(root: HTMLElement, date: number, start: number, end: number | null): Box | null {
  const hour = Math.floor(start / 60) * 60;
  const row = root.querySelector<HTMLElement>(`[data-hour="${hour}"]`);
  const cell = row?.querySelector<HTMLElement>(`[data-date="${date}"]`);
  const top = minuteY(root, start);
  if (!cell || top == null) {
    return null;
  }
  const rootRect = root.getBoundingClientRect();
  const cellRect = cell.getBoundingClientRect();
  const bottom = end == null ? null : minuteY(root, end);
  return {
    left: cellRect.left - rootRect.left + 4,
    top: top + 1,
    width: Math.max(0, cellRect.width - 8),
    height: bottom == null ? 22 : Math.max(18, bottom - top - 2)
  };
}

function measureNow(root: HTMLElement, today: number, days: number[], clock: number): NowLine | null {
  const day = days.find((item) => startOfLocalDay(item) === startOfLocalDay(today));
  if (day == null) {
    return null;
  }
  const date = new Date(clock);
  const minute = date.getHours() * 60 + date.getMinutes();
  const top = minuteY(root, minute);
  if (top == null) {
    return null;
  }
  const hour = Math.floor(minute / 60) * 60;
  const row =
    root.querySelector<HTMLElement>(`[data-hour="${hour}"]`) ??
    (minute % 60 === 0 ? root.querySelector<HTMLElement>(`[data-hour="${hour - 60}"]`) : null);
  const cell = row?.querySelector<HTMLElement>(`[data-date="${day}"]`);
  if (!cell) {
    return null;
  }
  const rootRect = root.getBoundingClientRect();
  const cellRect = cell.getBoundingClientRect();
  const label = `${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`;
  return { top, left: cellRect.left - rootRect.left, width: cellRect.width, label };
}

function measureSpans(root: HTMLElement, plans: Plan[]): SpanBox[] {
  return plans.flatMap((plan) => {
    if (!hasTimeRange(plan) || plan.scheduledAt == null || plan.dueAt == null) {
      return [];
    }
    const box = boxFor(root, startOfLocalDay(plan.scheduledAt), minutesOf(plan.scheduledAt), minutesOf(plan.dueAt));
    return box ? [{ plan, ...box }] : [];
  });
}
