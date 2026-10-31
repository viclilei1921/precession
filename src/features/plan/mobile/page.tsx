import { CaretLeftIcon, CaretRightIcon, MagnifyingGlassIcon, PlusIcon } from '@phosphor-icons/react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useEffect, useRef, useState } from 'react';
import type { Plan } from '@/bridge/plan';
import { planComplete, planGroupCreate, planUpdate } from '@/bridge/plan';
import { IconButton } from '@/components/button/icon';
import { PlanChip } from '@/features/plan/components/plan-chip';
import type { PlanPreset } from '@/features/plan/layout';
import { groupHue, plansOnDay, planWhen, WEEK_SHORT } from '@/features/plan/layout';
import { AppBar } from '@/layout/mobile/app-bar';
import { Fab } from '@/layout/mobile/fab';
import { useToastStore } from '@/layout/mobile/toast';
import { planGroupListQuery, planListQuery, refreshPlanViews } from '@/query/plan/query';
import {
  addLocalDays,
  formatDayHeading,
  formatMonthDay,
  isSameLocalDay,
  startOfLocalDay,
  startOfLocalWeek,
  toDateInputValue
} from '@/utils/day';
import { errorMessage } from '@/utils/error';
import { matchesQuery } from '@/utils/search';
import { PlanDetailSheet } from './detail-sheet';
import styles from './page.module.css';

function blankPreset(day: number): PlanPreset {
  return { date: toDateInputValue(day), end: '', allDay: true, time: '09:00', timeEnd: '' };
}

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
  const [finding, setFinding] = useState(false);
  const [query, setQuery] = useState('');
  const plansQuery = useQuery(planListQuery);
  const groupsQuery = useQuery(planGroupListQuery);
  const ensured = useRef(false);
  const groups = groupsQuery.data ?? [];
  const system = groups.find((group) => group.system) ?? groups.find((group) => group.name === '收集箱');
  const weekStart = startOfLocalWeek(cursor);
  const weekDays = Array.from({ length: 7 }, (_, index) => addLocalDays(weekStart, index));
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

  function shiftWeek(dir: number) {
    const next = addLocalDays(weekStart, dir * 7);
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
    closeSheet();
  }

  return (
    <section className={styles.page}>
      <AppBar title="计划" subtitle={subtitle}>
        <IconButton label="搜索计划" onClick={() => setFinding((current) => !current)}>
          <MagnifyingGlassIcon weight="regular" />
        </IconButton>
        <IconButton label="新建计划" onClick={openCreate}>
          <PlusIcon weight="regular" />
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
            <span>
              {formatMonthDay(weekDays[0] ?? weekStart)} – {formatMonthDay(weekDays[6] ?? weekStart)}
            </span>
            <button type="button" className={styles.today} onClick={goToday}>
              今天
            </button>
            <div className={styles.weekNav}>
              <IconButton label="上一周" size="compact" onClick={() => shiftWeek(-1)}>
                <CaretLeftIcon weight="regular" />
              </IconButton>
              <IconButton label="下一周" size="compact" onClick={() => shiftWeek(1)}>
                <CaretRightIcon weight="regular" />
              </IconButton>
            </div>
          </div>
          <div className={styles.days}>
            {weekDays.map((day, index) => {
              const count = plansOnDay(visible, day).length;
              return (
                <button
                  key={day}
                  type="button"
                  className={styles.day}
                  data-on={isSameLocalDay(day, selected) ? 'true' : undefined}
                  data-today={isSameLocalDay(day, today) ? 'true' : undefined}
                  onClick={() => {
                    setSelected(day);
                    setCursor(day);
                  }}
                >
                  <span>{WEEK_SHORT[index]}</span>
                  <strong>{new Date(day).getDate()}</strong>
                  <i data-busy={count > 0 ? 'true' : undefined} />
                </button>
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
      {sheetOpen ? null : <Fab label="新建计划" onClick={openCreate} />}
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
