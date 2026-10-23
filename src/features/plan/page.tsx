import { CaretDownIcon, CaretLeftIcon, CaretRightIcon, ListIcon, PlusIcon } from '@phosphor-icons/react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import clsx from 'clsx';
import type { MouseEventHandler } from 'react';
import { useEffect, useRef, useState } from 'react';
import type { Plan } from '@/bridge/plan';
import { planComplete, planGroupCreate, planUpdate } from '@/bridge/plan';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { Popover } from '@/components/popover';
import { rippleHost, useRipple } from '@/components/ripple';
import { usePageSearch } from '@/hooks/search';
import { planGroupListQuery, planListQuery, refreshPlanViews } from '@/query/plan/query';
import { addLocalDays, startOfLocalDay, startOfLocalMonth, startOfLocalWeek, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import { matchesQuery } from '@/utils/search';
import { AgendaView } from './components/agenda-view';
import { DrawerPanel, DrawerScrim } from './components/drawer-frame';
import { ListDrawer } from './components/list-drawer';
import { MonthView } from './components/month-view';
import { PlanDrawer } from './components/plan-drawer';
import { TimeGrid } from './components/time-grid';
import { YearView } from './components/year-view';
import type { PlanPreset, PlanView } from './layout';
import { atClock, endOfLocalMonth, groupHue, navLabel, shiftCursor, titleText, VIEW_LABEL } from './layout';
import styles from './page.module.css';

const VIEWS: PlanView[] = ['year', 'month', 'week', 'day', 'agenda'];

type Editor = {
  plan: Plan | null;
  preset: PlanPreset;
};

function blankPreset(day: number): PlanPreset {
  return { date: toDateInputValue(day), end: '', allDay: true, time: '09:00', timeEnd: '' };
}

/** 计划页：月周日日程年，清单和计划都从右侧抽屉打开 */
export function PlanPage() {
  const navigate = useNavigate();
  const search = useSearch({ from: '/plan' });
  const queryClient = useQueryClient();
  const query = usePageSearch('搜索计划');
  const today = startOfLocalDay();
  const [view, setView] = useState<PlanView>('month');
  const [cursor, setCursor] = useState(today);
  const [listId, setListId] = useState<string | null>(null);
  const [through, setThrough] = useState(() => endOfLocalMonth(new Date(new Date().getFullYear(), 11, 1).getTime()));
  const [menuAnchor, setMenuAnchor] = useState<HTMLElement | null>(null);
  const [listsOpen, setListsOpen] = useState(false);
  const [editor, setEditor] = useState<Editor | null>(null);
  const plansQuery = useQuery(planListQuery);
  const groupsQuery = useQuery(planGroupListQuery);
  const ensured = useRef(false);
  const pendingId = useRef<string | undefined>(undefined);
  const groups = groupsQuery.data ?? [];
  const system = groups.find((group) => group.system) ?? groups.find((group) => group.name === '收集箱');
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
  const [prevLabel, nextLabel] = navLabel(view);
  const canCreate = view !== 'year' && view !== 'agenda';
  const drawerOpen = listsOpen || editor != null;

  const movePlan = useMutation({
    mutationFn: ({ plan, day, start, end }: { plan: Plan; day: number; start: number; end: number | null }) => {
      const scheduledAt = atClock(day, padClock(start));
      const dueAt = end == null ? null : end >= 24 * 60 ? atClock(day, '23:59') : atClock(day, padClock(end));
      return planUpdate(plan.id, {
        scheduledAt,
        allDay: false,
        ...(dueAt == null ? (plan.dueAt != null ? { clearDueAt: true } : {}) : { dueAt })
      });
    },
    onSuccess: () => refreshPlanViews(queryClient)
  });

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
    if (search.create) {
      setListsOpen(false);
      setEditor({ plan: null, preset: blankPreset(today) });
      void navigate({ to: '/plan', search: {}, replace: true });
      return;
    }
    if (search.id) {
      pendingId.current = search.id;
      void navigate({ to: '/plan', search: {}, replace: true });
    }
  }, [navigate, search.create, search.id, today]);

  useEffect(() => {
    const id = pendingId.current;
    if (!id || !plansQuery.data) {
      return;
    }
    pendingId.current = undefined;
    const found = plansQuery.data.find((plan) => plan.id === id);
    if (found) {
      setListsOpen(false);
      setEditor({ plan: found, preset: blankPreset(found.scheduledAt ?? today) });
    }
  }, [plansQuery.data, today]);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (event.key !== 'Escape') {
        return;
      }
      setMenuAnchor(null);
      setListsOpen(false);
      setEditor(null);
    }
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, []);

  useEffect(() => {
    if (view !== 'agenda') {
      return;
    }
    const date = new Date(cursor);
    document.getElementById(`plan-month-${date.getFullYear()}-${date.getMonth()}`)?.scrollIntoView({ block: 'start' });
  }, [view, cursor]);

  function hueOf(plan: Plan) {
    const group = groups.find((item) => item.id === plan.groupId) ?? system;
    return groupHue(group?.color);
  }

  function closeDrawer() {
    setListsOpen(false);
    setEditor(null);
  }

  function openCreate(preset: PlanPreset) {
    if (!canCreate) {
      return;
    }
    setMenuAnchor(null);
    setListsOpen(false);
    setEditor({ plan: null, preset });
  }

  function openPlan(plan: Plan) {
    setMenuAnchor(null);
    setListsOpen(false);
    setEditor({ plan, preset: blankPreset(plan.scheduledAt ?? cursor) });
  }

  function shift(dir: number) {
    const next = shiftCursor(view, cursor, dir);
    setCursor(next);
    if (view === 'agenda') {
      const end = endOfLocalMonth(next);
      setThrough((current) => (end > current ? end : current));
    }
  }

  function saved(start: number | null) {
    setEditor(null);
    if (start != null) {
      setCursor(start);
      return;
    }
    if (view !== 'agenda') {
      setView('agenda');
    }
  }

  const weekStart = startOfLocalWeek(cursor);
  const weekDays = Array.from({ length: 7 }, (_, index) => addLocalDays(weekStart, index));
  const day = startOfLocalDay(cursor);

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>{titleText(view, cursor)}</h1>
        <div className={styles.tools}>
          {canCreate ? (
            <IconButton label="新建计划" variant="outlined" onClick={() => openCreate(blankPreset(cursor))}>
              <PlusIcon weight="regular" />
            </IconButton>
          ) : null}
          <Button variant="outlined" onClick={(event) => setMenuAnchor(menuAnchor ? null : event.currentTarget)}>
            {VIEW_LABEL[view]}
            <CaretDownIcon weight="regular" />
          </Button>
          <div className={styles.nav}>
            <IconButton label={prevLabel} shape="square" onClick={() => shift(-1)}>
              <CaretLeftIcon weight="regular" />
            </IconButton>
            <Button className={styles.navToday} variant="text" onClick={() => setCursor(today)}>
              今天
            </Button>
            <IconButton label={nextLabel} shape="square" onClick={() => shift(1)}>
              <CaretRightIcon weight="regular" />
            </IconButton>
          </div>
        </div>
        <IconButton
          className={styles.listButton}
          label="清单"
          variant="outlined"
          shape="rounded"
          expanded={listsOpen}
          onClick={() => {
            setEditor(null);
            setListsOpen((open) => !open);
          }}
        >
          <ListIcon weight="regular" />
        </IconButton>
      </header>
      <div className={styles.canvas}>
        {plansQuery.error ? <p className={styles.error}>{errorMessage(plansQuery.error)}</p> : null}
        {groupsQuery.error ? <p className={styles.error}>{errorMessage(groupsQuery.error)}</p> : null}
        {toggle.error ? <p className={styles.error}>{errorMessage(toggle.error)}</p> : null}
        {movePlan.error ? <p className={styles.error}>{errorMessage(movePlan.error)}</p> : null}
        {plansQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}
        {view === 'month' ? (
          <MonthView
            month={startOfLocalMonth(cursor)}
            today={today}
            plans={visible}
            hueOf={hueOf}
            onCreate={(time) => openCreate(blankPreset(time))}
            onOpen={openPlan}
            onToggle={(plan) => toggle.mutate(plan)}
          />
        ) : null}
        {view === 'week' ? (
          <TimeGrid
            days={weekDays}
            today={today}
            plans={visible}
            hueOf={hueOf}
            onOpenDay={(time) => {
              setCursor(time);
              setView('day');
            }}
            onCreateAllDay={(time) => openCreate(blankPreset(time))}
            onCreateRange={(time, from, to) =>
              openCreate({
                date: toDateInputValue(time),
                end: '',
                allDay: false,
                time: padClock(from),
                timeEnd: to >= 24 * 60 ? '23:59' : padClock(to)
              })
            }
            onOpen={openPlan}
            onToggle={(plan) => toggle.mutate(plan)}
            onMove={(plan, day, start, end) => movePlan.mutate({ plan, day, start, end })}
          />
        ) : null}
        {view === 'day' ? (
          <TimeGrid
            days={[day]}
            today={today}
            plans={visible}
            hueOf={hueOf}
            onOpenDay={setCursor}
            onCreateAllDay={(time) => openCreate(blankPreset(time))}
            onCreateRange={(time, from, to) =>
              openCreate({
                date: toDateInputValue(time),
                end: '',
                allDay: false,
                time: padClock(from),
                timeEnd: to >= 24 * 60 ? '23:59' : padClock(to)
              })
            }
            onOpen={openPlan}
            onToggle={(plan) => toggle.mutate(plan)}
            onMove={(plan, day, start, end) => movePlan.mutate({ plan, day, start, end })}
          />
        ) : null}
        {view === 'agenda' ? (
          <AgendaView
            through={through}
            plans={visible}
            hueOf={hueOf}
            onOpen={openPlan}
            onToggle={(plan) => toggle.mutate(plan)}
            onMore={() => setThrough((current) => endOfLocalMonth(addLocalDays(endOfLocalMonth(current), 1)))}
          />
        ) : null}
        {view === 'year' ? (
          <YearView
            yearStart={cursor}
            today={today}
            plans={visible}
            hueOf={hueOf}
            onOpenMonth={(month) => {
              setCursor(month);
              setView('month');
            }}
            onOpen={openPlan}
          />
        ) : null}
      </div>
      <Popover
        open={menuAnchor != null}
        anchorEl={menuAnchor}
        label="视图"
        className={styles.menu}
        anchorOrigin={{ horizontal: 'left', vertical: 'bottom' }}
        transformOrigin={{ horizontal: 'left', vertical: 'top' }}
        gap={8}
        onClose={() => setMenuAnchor(null)}
      >
        {VIEWS.map((item) => (
          <ViewChoice
            key={item}
            selected={item === view}
            onClick={() => {
              setView(item);
              setMenuAnchor(null);
              if (item === 'agenda') {
                const end = endOfLocalMonth(cursor);
                setThrough((current) => (end > current ? end : current));
              }
            }}
          >
            {VIEW_LABEL[item]}
          </ViewChoice>
        ))}
      </Popover>
      <DrawerScrim open={drawerOpen} onClose={closeDrawer} />
      <DrawerPanel open={listsOpen} title="清单" label="清单" onClose={closeDrawer}>
        <ListDrawer
          groups={groups}
          plans={plansQuery.data ?? []}
          systemId={system?.id}
          selectedId={listId}
          onPick={(id) => {
            setListId((current) => (current === id ? null : id));
            setListsOpen(false);
          }}
          onCreated={() => void refreshPlanViews(queryClient)}
        />
      </DrawerPanel>
      <DrawerPanel
        wide
        open={editor != null}
        title={editor?.plan ? '计划详情' : '新建计划'}
        label="计划"
        onClose={closeDrawer}
      >
        {editor ? (
          <PlanDrawer
            key={editor.plan?.id ?? `${editor.preset.date}-${editor.preset.time}-${editor.preset.timeEnd}`}
            plan={editor.plan}
            preset={editor.preset}
            groups={groups}
            onClose={closeDrawer}
            onSaved={saved}
          />
        ) : null}
      </DrawerPanel>
    </section>
  );
}

function padClock(minutes: number): string {
  const hour = Math.floor(minutes / 60);
  const minute = minutes % 60;
  return `${String(hour).padStart(2, '0')}:${String(minute).padStart(2, '0')}`;
}

function ViewChoice({
  selected,
  children,
  onClick
}: {
  selected: boolean;
  children: string;
  onClick: MouseEventHandler<HTMLButtonElement>;
}) {
  const ripple = useRipple();

  return (
    <button
      type="button"
      className={clsx(rippleHost, styles.menuItem)}
      data-on={selected ? 'true' : undefined}
      onClick={onClick}
      onPointerDown={ripple.onPointerDown}
      onKeyDown={ripple.onKeyDown}
    >
      {ripple.nodes}
      {children}
    </button>
  );
}
