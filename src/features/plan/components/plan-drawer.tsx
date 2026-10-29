import { CalendarBlankIcon, CaretDownIcon, CaretUpIcon, PlusIcon, TrashIcon } from '@phosphor-icons/react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import clsx from 'clsx';
import type { FormEvent, ReactNode } from 'react';
import { useRef, useState } from 'react';
import type { Plan, PlanGroup, PlanInput, PlanPatch, PlanStatus } from '@/bridge/plan';
import { planComplete, planCreate, planDelete, planUpdate } from '@/bridge/plan';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { Checkbox } from '@/components/checkbox';
import { DateField } from '@/components/date-field';
import type { MarkdownEditorHandle } from '@/components/markdown';
import { looseImageOwner, MarkdownField, mountLooseImages, retainBodyImages } from '@/components/markdown';
import { rippleHost, useRipple } from '@/components/ripple';
import { Scroll } from '@/components/scroll';
import { Select } from '@/components/select';
import { TextField } from '@/components/text-field';
import { refreshFiles } from '@/query/files/query';
import { refreshPlanViews } from '@/query/plan/query';
import { formatMonthDay, fromDateInputValue, startOfLocalDay, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import styles from '../drawer.module.css';
import type { PlanPreset } from '../layout';
import { atClock } from '../layout';
import { DrawerAdd } from './drawer-frame';

const PRIORITIES = [
  { value: 0, label: '普通' },
  { value: 1, label: '重要' },
  { value: 2, label: '紧急' }
] as const;

type StepDraft = {
  key: string;
  title: string;
  done: boolean;
};

type PlanDrawerProps = {
  plan: Plan | null;
  preset: PlanPreset;
  groups: PlanGroup[];
  onClose: () => void;
  onSaved: (start: number | null) => void;
};

/** 新建或编辑计划。平时只写标题、详情和步骤，其余先看结果 */
export function PlanDrawer({ plan, preset, groups, onClose, onSaved }: PlanDrawerProps) {
  const queryClient = useQueryClient();
  const markdownRef = useRef<MarkdownEditorHandle>(null);
  const [title, setTitle] = useState(plan?.title ?? '');
  const [body, setBody] = useState(plan?.body ?? '');
  const [groupId, setGroupId] = useState(
    plan?.groupId ?? groups.find((group) => group.system)?.id ?? groups[0]?.id ?? ''
  );
  const [priority, setPriority] = useState(plan?.priority ?? 0);
  const [date, setDate] = useState(plan ? toDateInputValue(plan.scheduledAt) : preset.date);
  const [end, setEnd] = useState(initialEnd(plan, preset));
  const [allDay, setAllDay] = useState(plan ? plan.allDay : preset.allDay);
  const [time, setTime] = useState(plan?.scheduledAt && !plan.allDay ? clockOf(plan.scheduledAt) : preset.time);
  const [timeEnd, setTimeEnd] = useState(plan?.dueAt && !plan.allDay ? clockOf(plan.dueAt) : preset.timeEnd);
  const [done, setDone] = useState(plan?.status === 'done');
  const [steps, setSteps] = useState<StepDraft[]>(
    () => plan?.steps.map((step) => ({ key: step.id, title: step.title, done: step.done })) ?? []
  );
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [whenOpen, setWhenOpen] = useState(false);
  const knownPriority = PRIORITIES.some((item) => item.value === priority);
  const today = startOfLocalDay();

  const save = useMutation({
    mutationFn: async () => {
      const fields = buildFields({
        title,
        body,
        groupId,
        priority,
        date,
        end,
        allDay,
        time,
        timeEnd,
        done: Boolean(plan) && done,
        steps
      });
      if (!fields.title) {
        throw new Error('标题不能为空');
      }
      if (!plan) {
        const created = await planCreate(fields.input);
        const body = fields.input.body ?? '';
        try {
          await mountLooseImages('plan', created.id, body);
        } catch (error) {
          await planDelete(created.id).catch(() => undefined);
          throw error;
        }
        markdownRef.current?.saved(body);
        return fields.start;
      }
      await planUpdate(plan.id, fields.patch);
      await retainBodyImages('plan', plan.id, fields.patch.body ?? '');
      markdownRef.current?.saved(fields.patch.body ?? '');
      if (done && plan.status !== 'done') {
        await planComplete(plan.id, plan.result);
      }
      return fields.start;
    },
    onSuccess: async (start) => {
      await refreshPlanViews(queryClient);
      await refreshFiles(queryClient);
      onSaved(start);
    }
  });
  const remove = useMutation({
    mutationFn: () => (plan ? planDelete(plan.id) : Promise.reject(new Error('计划不存在'))),
    onSuccess: async () => {
      await refreshPlanViews(queryClient);
      onClose();
    }
  });
  const pending = save.isPending || remove.isPending;

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!title.trim()) {
      return;
    }
    save.mutate();
  }

  function moveStep(index: number, delta: number) {
    setSteps((current) => {
      const target = index + delta;
      if (target < 0 || target >= current.length) {
        return current;
      }
      const next = current.slice();
      const [item] = next.splice(index, 1);
      if (!item) {
        return current;
      }
      next.splice(target, 0, item);
      return next;
    });
  }

  return (
    <form className={styles.form} onSubmit={submit}>
      <Scroll className={styles.sheet} axis="vertical">
        <div className={styles.main}>
          <div className={styles.titleRow}>
            <TextField
              label="标题"
              required
              maxLength={40}
              value={title}
              onChange={(event) => setTitle(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter') {
                  event.preventDefault();
                }
              }}
            />
            {plan ? (
              <button
                type="button"
                className={styles.done}
                aria-pressed={done}
                onClick={() => setDone((current) => !current)}
              >
                <span className={styles.check} data-on={done ? 'true' : undefined}>
                  <svg viewBox="0 0 16 16" aria-hidden="true">
                    <path d="M3.5 8.2 6.6 11.3 12.5 4.7" />
                  </svg>
                </span>
                已完成
              </button>
            ) : null}
          </div>
          <MarkdownField
            ref={markdownRef}
            label="详情"
            value={body}
            image={plan ? { owner: 'plan', ownerId: plan.id } : looseImageOwner}
            onChange={setBody}
          />
          <div className={styles.steps}>
            {steps.map((step, index) => (
              <div key={step.key} className={styles.step} data-done={step.done ? 'true' : undefined}>
                <button
                  type="button"
                  className={styles.check}
                  data-on={step.done ? 'true' : undefined}
                  aria-pressed={step.done}
                  aria-label={step.done ? '标为未完成' : '标为完成'}
                  onClick={() =>
                    setSteps((current) =>
                      current.map((item) => (item.key === step.key ? { ...item, done: !item.done } : item))
                    )
                  }
                >
                  <svg viewBox="0 0 16 16" aria-hidden="true">
                    <path d="M3.5 8.2 6.6 11.3 12.5 4.7" />
                  </svg>
                </button>
                <TextField
                  className={styles.stepTitle}
                  label="步骤"
                  value={step.title}
                  onChange={(event) =>
                    setSteps((current) =>
                      current.map((item) => (item.key === step.key ? { ...item, title: event.target.value } : item))
                    )
                  }
                  onKeyDown={(event) => {
                    if (event.key === 'Enter') {
                      event.preventDefault();
                    }
                  }}
                />
                <IconButton label="上移步骤" size="compact" disabled={index === 0} onClick={() => moveStep(index, -1)}>
                  <CaretUpIcon weight="regular" />
                </IconButton>
                <IconButton
                  label="下移步骤"
                  size="compact"
                  disabled={index === steps.length - 1}
                  onClick={() => moveStep(index, 1)}
                >
                  <CaretDownIcon weight="regular" />
                </IconButton>
                <IconButton
                  className={styles.remove}
                  label="删除步骤"
                  size="compact"
                  onClick={() => setSteps((current) => current.filter((item) => item.key !== step.key))}
                >
                  <TrashIcon weight="regular" />
                </IconButton>
              </div>
            ))}
            <DrawerAdd
              onClick={() => setSteps((current) => [...current, { key: crypto.randomUUID(), title: '', done: false }])}
            >
              <PlusIcon weight="regular" />
              添加步骤
            </DrawerAdd>
          </div>
          <Fact open={whenOpen} set={date !== ''} onClick={() => setWhenOpen((open) => !open)}>
            <CalendarBlankIcon weight="regular" />
            {describeWhen(date, end, allDay, time, timeEnd, today)}
          </Fact>
          {whenOpen ? (
            <div className={styles.editor}>
              <div className={styles.pair}>
                <DateField label="日期" value={date} onChange={setDate} />
                <DateField label="结束日期" value={end} onChange={setEnd} />
              </div>
              <Checkbox label="全天" checked={allDay} onChange={(event) => setAllDay(event.target.checked)} />
              {allDay ? null : (
                <div className={styles.pair}>
                  <TextField
                    label="开始"
                    type="time"
                    value={time}
                    step={60}
                    onChange={(event) => setTime(event.target.value)}
                  />
                  <TextField
                    label="结束"
                    type="time"
                    value={timeEnd}
                    step={60}
                    onChange={(event) => setTimeEnd(event.target.value)}
                  />
                </div>
              )}
              <p className={styles.hint}>日期留空，就放进未安排。结束日期留空，就只占开始这一天。</p>
            </div>
          ) : null}
          <div className={styles.pair}>
            <Select label="清单" value={groupId} onChange={(event) => setGroupId(event.target.value)}>
              {groups.map((item) => (
                <option key={item.id} value={item.id}>
                  {item.name}
                </option>
              ))}
            </Select>
            <Select label="优先级" value={priority} onChange={(event) => setPriority(Number(event.target.value))}>
              {knownPriority ? null : <option value={priority}>{priority}</option>}
              {PRIORITIES.map((item) => (
                <option key={item.value} value={item.value}>
                  {item.label}
                </option>
              ))}
            </Select>
          </div>
        </div>
      </Scroll>
      <div className={styles.foot}>
        {save.error ? <p className={styles.error}>{errorMessage(save.error)}</p> : null}
        {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
        <div className={styles.actions}>
          {plan ? (
            <Button
              variant="outlined"
              tone="destructive"
              disabled={pending}
              onClick={() => {
                if (!confirmDelete) {
                  setConfirmDelete(true);
                  return;
                }
                remove.mutate();
              }}
            >
              {confirmDelete ? '确认删除' : '删除'}
            </Button>
          ) : null}
          <span className={styles.spacer} />
          <Button variant="outlined" disabled={pending} onClick={onClose}>
            取消
          </Button>
          <Button type="submit" disabled={pending}>
            保存
          </Button>
        </div>
      </div>
    </form>
  );
}

function Fact({
  open,
  set,
  children,
  onClick
}: {
  open: boolean;
  set?: boolean;
  children: ReactNode;
  onClick: () => void;
}) {
  const ripple = useRipple();

  return (
    <button
      type="button"
      className={clsx(rippleHost, styles.fact)}
      data-set={set ? 'true' : undefined}
      data-open={open ? 'true' : undefined}
      aria-expanded={open}
      onClick={onClick}
      onPointerDown={ripple.onPointerDown}
      onKeyDown={ripple.onKeyDown}
    >
      {ripple.nodes}
      {children}
    </button>
  );
}

function describeWhen(
  date: string,
  end: string,
  allDay: boolean,
  time: string,
  timeEnd: string,
  today: number
): string {
  const start = fromDateInputValue(date);
  if (start == null) {
    return '未安排';
  }
  const finish = fromDateInputValue(end);
  const startText = dayText(start, today);
  const endText = finish != null && startOfLocalDay(finish) !== startOfLocalDay(start) ? dayText(finish, today) : '';
  if (allDay) {
    return endText ? `${startText} – ${endText}` : `${startText} 全天`;
  }
  const clock = time || '09:00';
  if (endText) {
    return timeEnd ? `${startText} ${clock} – ${endText} ${timeEnd}` : `${startText} ${clock} – ${endText}`;
  }
  return timeEnd ? `${startText} ${clock}–${timeEnd}` : `${startText} ${clock}`;
}

function dayText(day: number, today: number): string {
  const label = formatMonthDay(day);
  return startOfLocalDay(day) === startOfLocalDay(today) ? `${label} 今天` : label;
}

function initialEnd(plan: Plan | null, preset: PlanPreset): string {
  if (!plan || plan.scheduledAt == null || plan.dueAt == null) {
    return preset.end;
  }
  if (startOfLocalDay(plan.dueAt) === startOfLocalDay(plan.scheduledAt)) {
    return '';
  }
  return toDateInputValue(plan.dueAt);
}

function clockOf(time: number): string {
  const date = new Date(time);
  return `${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`;
}

function buildFields(input: {
  title: string;
  body: string;
  groupId: string;
  priority: number;
  date: string;
  end: string;
  allDay: boolean;
  time: string;
  timeEnd: string;
  done: boolean;
  steps: StepDraft[];
}): { title: string; start: number | null; input: PlanInput; patch: PlanPatch } {
  const title = input.title.trim();
  let startDay = fromDateInputValue(input.date);
  let endDay = fromDateInputValue(input.end);
  if (startDay != null && endDay != null && endDay < startDay) {
    const swap = startDay;
    startDay = endDay;
    endDay = swap;
  }
  if (endDay === startDay) {
    endDay = null;
  }
  const steps = input.steps
    .filter((step) => step.title.trim())
    .map((step) => ({ title: step.title.trim(), done: step.done }));
  let scheduledAt: number | null = null;
  let dueAt: number | null = null;
  if (startDay != null) {
    scheduledAt = input.allDay ? startDay : atClock(startDay, input.time || '09:00');
    if (input.allDay) {
      dueAt = endDay;
    } else if (input.timeEnd) {
      const dueDay = endDay ?? startDay;
      dueAt = atClock(dueDay, input.timeEnd);
      if (dueAt <= scheduledAt) {
        const swap = scheduledAt;
        scheduledAt = dueAt;
        dueAt = swap;
      }
      if (dueAt === scheduledAt) {
        dueAt = null;
      }
    } else {
      dueAt = endDay;
    }
  }
  const status: PlanStatus = input.done ? 'done' : startDay == null ? 'draft' : 'scheduled';
  const shared = {
    title,
    body: input.body.trim(),
    priority: input.priority,
    allDay: input.allDay || scheduledAt == null,
    status,
    steps
  };
  return {
    title,
    start: startDay,
    input: {
      ...shared,
      groupId: input.groupId || null,
      scheduledAt,
      dueAt
    },
    patch: {
      ...shared,
      ...(input.groupId ? { groupId: input.groupId } : {}),
      ...(scheduledAt == null ? { clearScheduledAt: true } : { scheduledAt }),
      ...(dueAt == null ? { clearDueAt: true } : { dueAt })
    }
  };
}
