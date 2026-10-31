import { CaretDownIcon, CaretLeftIcon, CaretRightIcon, ListIcon, MagnifyingGlassIcon } from '@phosphor-icons/react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import type { PointerEvent as ReactPointerEvent } from 'react';
import { useEffect, useMemo, useRef, useState } from 'react';
import type { Plan, PlanGroup } from '@/bridge/plan';
import { planComplete, planGroupCreate, planUpdate } from '@/bridge/plan';
import { IconButton } from '@/components/button/icon';
import { ListDrawer } from '@/features/plan/components/list-drawer';
import { PlanChip } from '@/features/plan/components/plan-chip';
import type { PlanPreset } from '@/features/plan/layout';
import { groupHue, monthWeeks, plansOnDay, planWhen, WEEK_SHORT } from '@/features/plan/layout';
import { dayFace } from '@/features/today/calendar/day-face';
import { AppBar } from '@/layout/mobile/app-bar';
import { Fab } from '@/layout/mobile/fab';
import { Sheet, useSheetClose } from '@/layout/mobile/sheet';
import { useToastStore } from '@/layout/mobile/toast';
import { calendarRangeQuery } from '@/query/calendar/query';
import { planGroupListQuery, planListQuery, refreshPlanViews } from '@/query/plan/query';
import {
  addLocalDays,
  addLocalMonths,
  formatDayHeading,
  formatMonthDay,
  isSameLocalDay,
  startOfLocalDay,
  startOfLocalMonth,
  startOfLocalWeek,
  toDateInputValue
} from '@/utils/day';
import { errorMessage } from '@/utils/error';
import { matchesQuery } from '@/utils/search';
import { AlmanacSheet } from './almanac-sheet';
import { PlanDetailSheet } from './detail-sheet';
import styles from './page.module.css';

function blankPreset(day: number): PlanPreset {
  return { date: toDateInputValue(day), end: '', allDay: true, time: '09:00', timeEnd: '' };
}

/** 一个月的周行。整周都落到下个月时不再画出。 */
function monthGrid(monthStart: number): number[][] {
  const month = new Date(monthStart).getMonth();
  return monthWeeks(monthStart).filter(
    (week, index) => index < 5 || week.some((day) => new Date(day).getMonth() === month)
  );
}

const HOLD_MS = 480;

type DayPress = {
  timer: number;
  day: number;
  fired: boolean;
  stop: () => void;
};

function sortDay(plans: Plan[]) {
  return [...plans].sort((left, right) => {
    const leftDone = left.status === 'done';
    const rightDone = right.status === 'done';
    if (leftDone !== rightDone) {
      return leftDone ? 1 : -1;
    }
    return (left.scheduledAt ?? 0) - (right.scheduledAt ?? 0);
  });
}

/** 移动端计划：一周日历、当天议程，详情留在地址栏里。 */
export function MobilePlanPage() {
  const navigate = useNavigate();
  const search = useSearch({ from: '/plan' });
  const queryClient = useQueryClient();
  const showToast = useToastStore((state) => state.show);
  const today = startOfLocalDay();
  const [cursor, setCursor] = useState(today);
  const [selected, setSelected] = useState(today);
  const [listId, setListId] = useState<string | null>(null);
  const [listsOpen, setListsOpen] = useState(false);
  const [monthOpen, setMonthOpen] = useState(false);
  const [almanacDay, setAlmanacDay] = useState<number | null>(null);
  const [finding, setFinding] = useState(false);
  const [query, setQuery] = useState('');
  const plansQuery = useQuery(planListQuery);
  const groupsQuery = useQuery(planGroupListQuery);
  const ensured = useRef(false);
  const pressRef = useRef<DayPress | null>(null);
  const skipClick = useRef(false);
  const groups = groupsQuery.data ?? [];
  const system = groups.find((group) => group.system) ?? groups.find((group) => group.name === '收集箱');
  const gridAnchor = monthOpen ? cursor : selected;
  const monthStart = startOfLocalMonth(gridAnchor);
  const weeks = monthGrid(monthStart);
  const activeRow = Math.max(
    0,
    weeks.findIndex((week) => week.some((day) => isSameLocalDay(day, selected)))
  );
  const shownWeek = weeks[activeRow] ?? [selected];
  const monthDate = new Date(monthStart);
  const rangeFrom = toDateInputValue(weeks[0]?.[0] ?? monthStart);
  const rangeTo = toDateInputValue(weeks[weeks.length - 1]?.[6] ?? monthStart);
  const calendarQuery = useQuery(calendarRangeQuery(rangeFrom, rangeTo));
  const calendarDays = useMemo(
    () => new Map((calendarQuery.data?.days ?? []).map((day) => [day.date, day])),
    [calendarQuery.data]
  );
  const rangeLabel = monthOpen
    ? `${monthDate.getFullYear()}年${monthDate.getMonth() + 1}月`
    : `${formatMonthDay(shownWeek[0] ?? selected)} – ${formatMonthDay(shownWeek[6] ?? selected)}`;
  const listed = (plansQuery.data ?? []).filter((plan) => matchesQuery(query, plan.title, plan.body, plan.result));
  const visible = listed.filter((plan) => {
    if (!listId) {
      return true;
    }
    if (plan.groupId === listId) {
      return true;
    }
    return listId === system?.id && plan.groupId == null;
  });
  const dayPlans = sortDay(plansOnDay(visible, selected));
  const openPlan = search.id ? (plansQuery.data?.find((plan) => plan.id === search.id) ?? null) : null;
  const sheetOpen = Boolean(search.create || search.id);
  const selectedDate = new Date(selected);
  const subtitle = isSameLocalDay(selected, today)
    ? `${selectedDate.getFullYear()}年${selectedDate.getMonth() + 1}月 · 今天`
    : formatDayHeading(selected);

  const toggle = useMutation({
    mutationFn: (plan: Plan) =>
      plan.status === 'done'
        ? planUpdate(plan.id, { status: plan.scheduledAt == null ? 'draft' : 'scheduled' })
        : planComplete(plan.id, plan.result || ''),
    onSuccess: () => refreshPlanViews(queryClient)
  });

  useEffect(() => {
    const loaded = groupsQuery.data;
    if (!loaded || ensured.current || loaded.some((group) => group.system || group.name === '收集箱')) {
      return;
    }
    ensured.current = true;
    void planGroupCreate({ name: '收集箱', color: '175', system: true, sort: 0 }).then(() =>
      refreshPlanViews(queryClient)
    );
  }, [groupsQuery.data, queryClient]);

  useEffect(() => {
    if (!search.id || plansQuery.isPending) {
      return;
    }
    if (!plansQuery.data?.some((plan) => plan.id === search.id)) {
      showToast('这条计划已经不在了');
      void navigate({ to: '/plan', search: {}, replace: true });
    }
  }, [navigate, plansQuery.data, plansQuery.isPending, search.id, showToast]);

  function shift(dir: number) {
    if (monthOpen) {
      const nextMonth = addLocalMonths(monthStart, dir);
      const next = new Date(nextMonth);
      const now = new Date(today);
      const sameMonth = now.getFullYear() === next.getFullYear() && now.getMonth() === next.getMonth();
      const day = sameMonth ? today : nextMonth;
      setCursor(day);
      setSelected(day);
      return;
    }
    const next = addLocalDays(startOfLocalWeek(selected), dir * 7);
    setCursor(next);
    setSelected(next);
  }

  function goToday() {
    setCursor(today);
    setSelected(today);
  }

  function closeSheet() {
    void navigate({ to: '/plan', search: {} });
  }

  function openCreate() {
    void navigate({ to: '/plan', search: { create: true } });
  }

  function saved(start: number | null) {
    if (start != null) {
      const day = startOfLocalDay(start);
      setCursor(day);
      setSelected(day);
    }
  }

  useEffect(() => () => pressRef.current?.stop(), []);

  function pressStart(day: number, event: ReactPointerEvent<HTMLButtonElement>) {
    if (event.button !== 0) {
      return;
    }
    pressRef.current?.stop();
    const pointerId = event.pointerId;
    const originX = event.clientX;
    const originY = event.clientY;
    const press: DayPress = { timer: 0, day, fired: false, stop: () => undefined };
    function end(open: boolean) {
      window.clearTimeout(press.timer);
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
      window.removeEventListener('pointercancel', cancel);
      if (pressRef.current !== press) {
        return;
      }
      pressRef.current = null;
      if (open && press.fired) {
        skipClick.current = true;
        setAlmanacDay(press.day);
      }
    }
    function move(ev: PointerEvent) {
      if (ev.pointerId !== pointerId || press.fired) {
        return;
      }
      if (Math.hypot(ev.clientX - originX, ev.clientY - originY) > 10) {
        end(false);
      }
    }
    function up(ev: PointerEvent) {
      if (ev.pointerId !== pointerId) {
        return;
      }
      end(true);
    }
    function cancel(ev: PointerEvent) {
      if (ev.pointerId !== pointerId) {
        return;
      }
      end(false);
    }
    press.stop = () => end(false);
    press.timer = window.setTimeout(() => {
      press.fired = true;
    }, HOLD_MS);
    pressRef.current = press;
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
    window.addEventListener('pointercancel', cancel);
  }

  function pickDay(day: number) {
    if (skipClick.current) {
      skipClick.current = false;
      return;
    }
    setSelected(day);
    setCursor(day);
  }

  return (
    <section className={styles.page}>
      <AppBar title="计划" subtitle={subtitle}>
        <IconButton label="搜索计划" onClick={() => setFinding((current) => !current)}>
          <MagnifyingGlassIcon weight="regular" />
        </IconButton>
        <IconButton label="设置清单" expanded={listsOpen} onClick={() => setListsOpen(true)}>
          <ListIcon weight="regular" />
        </IconButton>
      </AppBar>
      {finding ? (
        <form
          className={styles.search}
          onSubmit={(event) => {
            event.preventDefault();
          }}
        >
          <input
            type="search"
            value={query}
            placeholder="搜索计划标题或内容"
            aria-label="搜索计划"
            onChange={(event) => setQuery(event.target.value)}
          />
          <button
            type="button"
            onClick={() => {
              setQuery('');
              setFinding(false);
            }}
          >
            取消
          </button>
        </form>
      ) : null}
      <div className={styles.body}>
        <section className={styles.week}>
          <div className={styles.weekHead}>
            <button
              type="button"
              className={styles.range}
              aria-expanded={monthOpen}
              aria-label={`${rangeLabel}，${monthOpen ? '收起月历' : '展开月历'}`}
              onClick={() => setMonthOpen((open) => !open)}
            >
              <span>{rangeLabel}</span>
              <CaretDownIcon weight="bold" />
            </button>
            <button type="button" className={styles.today} onClick={goToday}>
              今天
            </button>
            <div className={styles.weekNav}>
              <IconButton label={monthOpen ? '上一月' : '上一周'} size="compact" onClick={() => shift(-1)}>
                <CaretLeftIcon weight="regular" />
              </IconButton>
              <IconButton label={monthOpen ? '下一月' : '下一周'} size="compact" onClick={() => shift(1)}>
                <CaretRightIcon weight="regular" />
              </IconButton>
            </div>
          </div>
          <div className={styles.dayNames}>
            {WEEK_SHORT.map((label) => (
              <span key={label}>{label}</span>
            ))}
          </div>
          <div className={styles.weeks}>
            {weeks.map((week, index) => {
              const shown = monthOpen || index === activeRow;
              return (
                <div key={week[0]} className={styles.weekClip} data-show={shown ? 'true' : undefined} inert={!shown}>
                  <div className={styles.weekRow}>
                    {week.map((day) => {
                      const outside = monthOpen && new Date(day).getMonth() !== monthDate.getMonth();
                      const on = isSameLocalDay(day, selected);
                      const face = dayFace(calendarDays.get(toDateInputValue(day)));
                      return (
                        <button
                          key={day}
                          type="button"
                          className={styles.day}
                          data-on={on ? 'true' : undefined}
                          data-today={isSameLocalDay(day, today) ? 'true' : undefined}
                          data-out={outside ? 'true' : undefined}
                          aria-pressed={on}
                          aria-label={face.text ? `${formatDayHeading(day)} ${face.text}` : formatDayHeading(day)}
                          onPointerDown={(event) => pressStart(day, event)}
                          onContextMenu={(event) => event.preventDefault()}
                          onClick={() => pickDay(day)}
                        >
                          <strong>{new Date(day).getDate()}</strong>
                          <span className={styles.lunar} data-kind={face.text ? face.kind : undefined}>
                            {face.text}
                          </span>
                          <i data-busy={plansOnDay(visible, day).length > 0 ? 'true' : undefined} />
                        </button>
                      );
                    })}
                  </div>
                </div>
              );
            })}
          </div>
          <p className={styles.weekLine}>
            {isSameLocalDay(selected, today) ? '今天' : formatMonthDay(selected)}有 {dayPlans.length} 项安排
          </p>
        </section>
        <div className={styles.filters} role="toolbar" aria-label="清单">
          <button type="button" data-on={listId == null ? 'true' : undefined} onClick={() => setListId(null)}>
            全部
          </button>
          {groups.map((group) => (
            <button
              key={group.id}
              type="button"
              data-on={listId === group.id ? 'true' : undefined}
              onClick={() => setListId((current) => (current === group.id ? null : group.id))}
            >
              {group.name}
            </button>
          ))}
        </div>
        <div className={styles.dayHead}>
          <strong>{formatDayHeading(selected)}</strong>
          <span>{dayPlans.length} 项</span>
        </div>
        {plansQuery.error ? <p className={styles.error}>{errorMessage(plansQuery.error)}</p> : null}
        {dayPlans.length === 0 && !plansQuery.isPending ? (
          <p className={styles.empty}>这一天还没有安排。点右下角新建一条。</p>
        ) : (
          <div className={styles.list}>
            {dayPlans.map((plan) => (
              <PlanChip
                key={plan.id}
                plan={plan}
                hue={groupHue(groups.find((group) => group.id === plan.groupId)?.color)}
                time={planWhen(plan)}
                onOpen={(item) => void navigate({ to: '/plan', search: { id: item.id } })}
                onToggle={(item) => toggle.mutate(item)}
              />
            ))}
          </div>
        )}
      </div>
      {sheetOpen || listsOpen || almanacDay != null ? null : <Fab label="新建计划" onClick={openCreate} />}
      {almanacDay != null ? <AlmanacSheet day={almanacDay} onClose={() => setAlmanacDay(null)} /> : null}
      {listsOpen ? (
        <ListSheet
          groups={groups}
          plans={plansQuery.data ?? []}
          systemId={system?.id}
          selectedId={listId}
          onClose={() => setListsOpen(false)}
          onPick={(id) => setListId((current) => (current === id ? null : id))}
          onCreated={() => void refreshPlanViews(queryClient)}
          onDeleted={(id) => {
            setListId((current) => (current === id ? null : current));
            void refreshPlanViews(queryClient);
          }}
        />
      ) : null}
      {sheetOpen && (!search.id || openPlan) ? (
        <PlanDetailSheet
          plan={search.id ? openPlan : null}
          preset={blankPreset(selected)}
          groups={groups}
          onClose={closeSheet}
          onSaved={saved}
        />
      ) : null}
    </section>
  );
}

function ListSheet({
  groups,
  plans,
  systemId,
  selectedId,
  onClose,
  onPick,
  onCreated,
  onDeleted
}: {
  groups: PlanGroup[];
  plans: Plan[];
  systemId?: string;
  selectedId: string | null;
  onClose: () => void;
  onPick: (id: string) => void;
  onCreated: () => void;
  onDeleted: (id: string) => void;
}) {
  return (
    <Sheet title="设置清单" onClose={onClose}>
      <ListSheetBody
        groups={groups}
        plans={plans}
        systemId={systemId}
        selectedId={selectedId}
        onClose={onClose}
        onPick={onPick}
        onCreated={onCreated}
        onDeleted={onDeleted}
      />
    </Sheet>
  );
}

function ListSheetBody({
  groups,
  plans,
  systemId,
  selectedId,
  onClose,
  onPick,
  onCreated,
  onDeleted
}: {
  groups: PlanGroup[];
  plans: Plan[];
  systemId?: string;
  selectedId: string | null;
  onClose: () => void;
  onPick: (id: string) => void;
  onCreated: () => void;
  onDeleted: (id: string) => void;
}) {
  const requestClose = useSheetClose();

  return (
    <ListDrawer
      flush
      groups={groups}
      plans={plans}
      systemId={systemId}
      selectedId={selectedId}
      onPick={(id) => {
        onPick(id);
        if (requestClose) {
          requestClose();
          return;
        }
        onClose();
      }}
      onCreated={onCreated}
      onDeleted={onDeleted}
    />
  );
}
