import { PlusIcon, TrashIcon } from '@phosphor-icons/react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useEffect, useRef, useState } from 'react';
import type { Plan, PlanInput, PlanPatch, PlanStatus } from '@/bridge/plan';
import { planCreate, planDelete, planUpdate } from '@/bridge/plan';
import { refreshPlanViews } from '@/shared/data/plan/query';
import { fromDateInputValue, startOfLocalDay, toDateInputValue } from '@/shared/lib/day';
import { errorMessage } from '@/shared/lib/error';
import styles from './plan-form.module.css';

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

type PlanFormProps = {
  plan: Plan | null;
  onClose: () => void;
};

export function PlanForm({ plan, onClose }: PlanFormProps) {
  const panelRef = useRef<HTMLFormElement>(null);
  const returnFocus = useRef<HTMLElement | null>(null);
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;
  const queryClient = useQueryClient();
  const done = plan?.status === 'done';
  const [title, setTitle] = useState(plan?.title ?? '');
  const [body, setBody] = useState(plan?.body ?? '');
  const [status, setStatus] = useState<PlanStatus>(plan?.status === 'inbox' ? 'inbox' : 'scheduled');
  const [priority, setPriority] = useState(plan?.priority ?? 0);
  const [scheduled, setScheduled] = useState(
    toDateInputValue(plan?.scheduledAt ?? (plan?.status === 'inbox' ? null : startOfLocalDay()))
  );
  const [due, setDue] = useState(toDateInputValue(plan?.dueAt ?? null));
  const [steps, setSteps] = useState<StepDraft[]>(
    () => plan?.steps.map((step) => ({ key: step.id, title: step.title, done: step.done })) ?? []
  );
  const [confirmDelete, setConfirmDelete] = useState(false);

  const save = useMutation({
    mutationFn: (payload: { kind: 'create'; input: PlanInput } | { kind: 'update'; patch: PlanPatch }) =>
      payload.kind === 'create' ? planCreate(payload.input) : planUpdate(plan?.id ?? '', payload.patch),
    onSuccess: async () => {
      await refreshPlanViews(queryClient);
      onCloseRef.current();
    }
  });
  const remove = useMutation({
    mutationFn: () => {
      if (!plan) {
        return Promise.reject(new Error('计划不存在'));
      }
      return planDelete(plan.id);
    },
    onSuccess: async () => {
      await refreshPlanViews(queryClient);
      onCloseRef.current();
    }
  });
  const pending = save.isPending || remove.isPending;
  const knownPriority = PRIORITIES.some((item) => item.value === priority);

  useEffect(() => {
    returnFocus.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    panelRef.current?.focus();

    function onKey(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        onCloseRef.current();
      }
    }

    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('keydown', onKey);
      returnFocus.current?.focus();
    };
  }, []);

  function formFields() {
    return {
      title,
      body,
      status: (done ? 'done' : status) as PlanStatus,
      priority,
      scheduledAt: status === 'inbox' && !done ? null : (fromDateInputValue(scheduled) ?? startOfLocalDay()),
      dueAt: fromDateInputValue(due),
      result: plan?.result ?? '',
      locked: plan?.locked ?? false,
      highlight: plan?.highlight ?? false,
      steps: steps.filter((step) => step.title.trim()).map((step) => ({ title: step.title.trim(), done: step.done }))
    };
  }

  return (
    <div className={styles.backdrop}>
      <button type="button" className={styles.scrim} aria-label="关闭" onClick={onClose} />
      <form
        ref={panelRef}
        className={styles.dialog}
        role="dialog"
        aria-modal="true"
        aria-label={plan ? '编辑计划' : '新建计划'}
        tabIndex={-1}
        onSubmit={(event) => {
          event.preventDefault();
          const fields = formFields();
          if (!plan) {
            save.mutate({ kind: 'create', input: fields });
            return;
          }
          const { scheduledAt, dueAt, ...rest } = fields;
          save.mutate({
            kind: 'update',
            patch: {
              ...rest,
              ...(scheduledAt != null ? { scheduledAt } : {}),
              ...(dueAt != null ? { dueAt } : {})
            }
          });
        }}
      >
        <h2 className={styles.title}>{plan ? '编辑计划' : '新建计划'}</h2>
        <label className={styles.field}>
          标题
          <input value={title} required onChange={(event) => setTitle(event.target.value)} />
        </label>
        <label className={styles.field}>
          备注
          <textarea value={body} onChange={(event) => setBody(event.target.value)} />
        </label>
        <div className={styles.pair}>
          <label className={styles.field}>
            优先级
            <select value={priority} onChange={(event) => setPriority(Number(event.target.value))}>
              {knownPriority ? null : <option value={priority}>{priority}</option>}
              {PRIORITIES.map((item) => (
                <option key={item.value} value={item.value}>
                  {item.label}
                </option>
              ))}
            </select>
          </label>
          <label className={styles.field} htmlFor="plan-status">
            状态
            {done ? (
              <input id="plan-status" value="已完成" disabled />
            ) : (
              <select id="plan-status" value={status} onChange={(event) => setStatus(event.target.value as PlanStatus)}>
                <option value="inbox">收集箱</option>
                <option value="scheduled">已安排</option>
              </select>
            )}
          </label>
        </div>
        {done || status === 'scheduled' ? (
          <div className={styles.pair}>
            <label className={styles.field}>
              安排时间
              <input type="date" value={scheduled} onChange={(event) => setScheduled(event.target.value)} />
            </label>
            <label className={styles.field}>
              截止时间
              <input type="date" value={due} onChange={(event) => setDue(event.target.value)} />
            </label>
          </div>
        ) : (
          <label className={styles.field}>
            截止时间
            <input type="date" value={due} onChange={(event) => setDue(event.target.value)} />
          </label>
        )}
        <div className={styles.steps}>
          <span className={styles.field}>步骤</span>
          {steps.map((step) => (
            <div key={step.key} className={styles.step}>
              <input
                type="checkbox"
                checked={step.done}
                aria-label="步骤完成"
                onChange={(event) =>
                  setSteps((current) =>
                    current.map((item) => (item.key === step.key ? { ...item, done: event.target.checked } : item))
                  )
                }
              />
              <input
                type="text"
                value={step.title}
                aria-label="步骤标题"
                onChange={(event) =>
                  setSteps((current) =>
                    current.map((item) => (item.key === step.key ? { ...item, title: event.target.value } : item))
                  )
                }
              />
              <button
                type="button"
                className={styles.remove}
                aria-label="删除步骤"
                onClick={() => setSteps((current) => current.filter((item) => item.key !== step.key))}
              >
                <TrashIcon className={styles.icon} weight="regular" />
              </button>
            </div>
          ))}
          <button
            type="button"
            className={styles.add}
            onClick={() => setSteps((current) => [...current, { key: crypto.randomUUID(), title: '', done: false }])}
          >
            <PlusIcon className={styles.icon} weight="regular" />
            添加步骤
          </button>
        </div>
        {save.error ? <p className={styles.error}>{errorMessage(save.error)}</p> : null}
        {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
        <div className={styles.actions}>
          {plan ? (
            <button
              type="button"
              className={styles.danger}
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
            </button>
          ) : null}
          <span className={styles.spacer} />
          <button type="button" className={styles.ghost} disabled={pending} onClick={onClose}>
            取消
          </button>
          <button type="submit" className={styles.primary} disabled={pending}>
            保存
          </button>
        </div>
      </form>
    </div>
  );
}
