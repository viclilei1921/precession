import { CalendarBlank, ListChecks, Plus, Tray } from '@phosphor-icons/react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useEffect, useRef, useState } from 'react';
import type { Plan } from '@/bridge/plan';
import { planUpdate } from '@/bridge/plan';
import { CompleteDialog } from '@/shared/data/plan/complete-dialog';
import { groupTodo, markedDays, planMeta, plansOnDay, weekCompletion } from '@/shared/data/plan/group';
import { toPlanInput } from '@/shared/data/plan/input';
import { PlanCheck } from '@/shared/data/plan/plan-check';
import { planListQuery, refreshPlanViews } from '@/shared/data/plan/query';
import { formatMonthDay, fromDateInputValue, startOfLocalDay, startOfLocalMonth } from '@/shared/lib/day';
import { errorMessage } from '@/shared/lib/error';
import { MonthBoard } from './components/month-board';
import { PlanForm } from './components/plan-form';
import styles from './page.module.css';

type PlanView = 'todo' | 'inbox' | 'calendar';
type Editing = Plan | 'new' | null;

export function PlanPage() {
  const navigate = useNavigate();
  const search = useSearch({ from: '/plan' });
  const queryClient = useQueryClient();
  const plansQuery = useQuery(planListQuery);
  const plans = plansQuery.data ?? [];
  const todayStart = startOfLocalDay();
  const [view, setView] = useState<PlanView>('todo');
  const [month, setMonth] = useState(() => startOfLocalMonth());
  const [selectedDay, setSelectedDay] = useState(todayStart);
  const [editing, setEditing] = useState<Editing>(null);
  const [completing, setCompleting] = useState<Plan | null>(null);
  const pendingId = useRef<string | undefined>(undefined);

  const schedule = useMutation({
    mutationFn: ({ plan, day }: { plan: Plan; day: number }) =>
      planUpdate(plan.id, toPlanInput(plan, { status: 'scheduled', scheduledAt: day })),
    onSuccess: () => refreshPlanViews(queryClient)
  });

  useEffect(() => {
    if (search.create) {
      setEditing('new');
      pendingId.current = undefined;
      void navigate({ to: '/plan', search: {}, replace: true });
      return;
    }
    if (search.id) {
      pendingId.current = search.id;
      void navigate({ to: '/plan', search: {}, replace: true });
    }
  }, [navigate, search.create, search.id]);

  useEffect(() => {
    const id = pendingId.current;
    if (!id || !plansQuery.data) {
      return;
    }
    pendingId.current = undefined;
    const found = plansQuery.data.find((plan) => plan.id === id);
    if (found) {
      setEditing(found);
    }
  }, [plansQuery.data]);

  function openDay(time: number) {
    setSelectedDay(startOfLocalDay(time));
    setMonth(startOfLocalMonth(time));
    setView('calendar');
  }

  const groups = groupTodo(plans, todayStart);
  const inbox = plans.filter((plan) => plan.status === 'inbox').sort((left, right) => right.createdAt - left.createdAt);
  const week = weekCompletion(plans, todayStart);
  const weekRate = week.total === 0 ? 0 : Math.round((week.done / week.total) * 100);
  const marks = markedDays(plans);
  const dayPlans = plansOnDay(plans, selectedDay);
  const editingPlan = editing === 'new' ? null : editing;

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>计划</h1>
        <p className={styles.hint}>面向未来的前置记录 · 完成即记入轨迹</p>
        <div className={styles.segments} role="tablist" aria-label="计划视图">
          <button
            type="button"
            className={styles.segment}
            data-on={view === 'todo' ? 'true' : undefined}
            onClick={() => setView('todo')}
          >
            <ListChecks className={styles.icon} weight="regular" />
            待办
          </button>
          <button
            type="button"
            className={styles.segment}
            data-on={view === 'inbox' ? 'true' : undefined}
            onClick={() => setView('inbox')}
          >
            <Tray className={styles.icon} weight="regular" />
            收集箱 {inbox.length}
          </button>
          <button
            type="button"
            className={styles.segment}
            data-on={view === 'calendar' ? 'true' : undefined}
            onClick={() => setView('calendar')}
          >
            <CalendarBlank className={styles.icon} weight="regular" />
            日历
          </button>
        </div>
        <button type="button" className={styles.primary} onClick={() => setEditing('new')}>
          <Plus className={styles.icon} weight="regular" />
          新建计划
        </button>
      </header>

      {plansQuery.error ? <p className={styles.error}>{errorMessage(plansQuery.error)}</p> : null}
      {schedule.error ? <p className={styles.error}>{errorMessage(schedule.error)}</p> : null}
      {plansQuery.isPending ? <p className={styles.note}>正在读取…</p> : null}

      {view === 'todo' ? (
        <div className={styles.split}>
          <div className={styles.card}>
            {groups.overdue.length + groups.today.length + groups.tomorrow.length + groups.later.length === 0 ? (
              <p className={styles.empty}>没有待办。新建一条，或从收集箱排期。</p>
            ) : (
              <>
                <PlanGroup
                  title="逾期"
                  plans={groups.overdue}
                  todayStart={todayStart}
                  onComplete={setCompleting}
                  onOpen={setEditing}
                />
                <PlanGroup
                  title={`今天 · ${formatMonthDay(todayStart)}`}
                  plans={groups.today}
                  todayStart={todayStart}
                  onComplete={setCompleting}
                  onOpen={setEditing}
                />
                <PlanGroup
                  title="明天"
                  plans={groups.tomorrow}
                  todayStart={todayStart}
                  onComplete={setCompleting}
                  onOpen={setEditing}
                />
                <PlanGroup
                  title="以后"
                  plans={groups.later}
                  todayStart={todayStart}
                  onComplete={setCompleting}
                  onOpen={setEditing}
                />
              </>
            )}
          </div>
          <div className={styles.stack}>
            <div className={styles.card}>
              <MonthBoard month={month} selected={selectedDay} marked={marks} onMonth={setMonth} onSelect={openDay} />
              <p className={styles.note}>有计划的日期有圆点。点某一天看当天。</p>
            </div>
            <div className={styles.card}>
              <span className={styles.statValue}>{weekRate}%</span>
              <span className={styles.statLabel}>
                本周完成率 · {week.done}/{week.total}
              </span>
              <div className={styles.progress}>
                <i style={{ width: `${weekRate}%` }} />
              </div>
            </div>
            <button type="button" className={styles.linkCard} onClick={() => setView('inbox')}>
              <span className={styles.cardTitle}>收集箱</span>
              <span className={styles.note}>{inbox.length} 条未排期，选一天即可排上。</span>
            </button>
          </div>
        </div>
      ) : null}

      {view === 'inbox' ? (
        <div className={styles.card}>
          {inbox.length === 0 ? <p className={styles.empty}>收集箱是空的。先随手记下，之后再排期。</p> : null}
          {inbox.map((plan) => (
            <div key={plan.id} className={styles.inboxRow}>
              <button type="button" className={styles.inboxTitle} onClick={() => setEditing(plan)}>
                {plan.title}
              </button>
              <label className={styles.schedule}>
                安排到
                <input
                  type="date"
                  aria-label={`把${plan.title}安排到`}
                  disabled={schedule.isPending}
                  onChange={(event) => {
                    const day = fromDateInputValue(event.target.value);
                    event.target.value = '';
                    if (day != null) {
                      schedule.mutate({ plan, day });
                    }
                  }}
                />
              </label>
            </div>
          ))}
        </div>
      ) : null}

      {view === 'calendar' ? (
        <div className={styles.split}>
          <div className={styles.card}>
            <MonthBoard
              month={month}
              selected={selectedDay}
              marked={marks}
              onMonth={setMonth}
              onSelect={(time) => {
                setSelectedDay(startOfLocalDay(time));
                setMonth(startOfLocalMonth(time));
              }}
            />
          </div>
          <div className={styles.card}>
            <h2 className={styles.cardTitle}>{formatMonthDay(selectedDay)}</h2>
            {dayPlans.length === 0 ? <p className={styles.empty}>这一天没有计划。</p> : null}
            {dayPlans.map((plan) => (
              <PlanCheck
                key={plan.id}
                plan={plan}
                meta={planMeta(plan, todayStart)}
                onComplete={setCompleting}
                onOpen={setEditing}
              />
            ))}
          </div>
        </div>
      ) : null}

      {editing !== null ? (
        <PlanForm key={editing === 'new' ? 'new' : editing.id} plan={editingPlan} onClose={() => setEditing(null)} />
      ) : null}
      <CompleteDialog plan={completing} onClose={() => setCompleting(null)} />
    </section>
  );
}

function PlanGroup({
  title,
  plans,
  todayStart,
  onComplete,
  onOpen
}: {
  title: string;
  plans: Plan[];
  todayStart: number;
  onComplete: (plan: Plan) => void;
  onOpen: (plan: Plan) => void;
}) {
  if (plans.length === 0) {
    return null;
  }
  return (
    <section>
      <h2 className={styles.cardTitle}>
        {title}
        {title === '逾期' ? <span className={styles.extra}>{plans.length}</span> : null}
      </h2>
      {plans.map((plan) => (
        <PlanCheck
          key={plan.id}
          plan={plan}
          meta={planMeta(plan, todayStart)}
          onComplete={onComplete}
          onOpen={onOpen}
        />
      ))}
    </section>
  );
}
