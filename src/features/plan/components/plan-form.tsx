import { PlusIcon, TrashIcon } from '@phosphor-icons/react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useRef, useState } from 'react';
import type { Plan, PlanInput, PlanPatch, PlanStatus } from '@/bridge/plan';
import { planCreate, planDelete, planUpdate } from '@/bridge/plan';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { Checkbox } from '@/components/checkbox';
import { DateField } from '@/components/date-field';
import { Modal } from '@/components/modal';
import { Select } from '@/components/select';
import { TextField } from '@/components/text-field';
import { Textarea } from '@/components/textarea';
import { refreshPlanViews } from '@/query/plan/query';
import { fromDateInputValue, startOfLocalDay, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';
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
    <Modal wide title={plan ? '编辑计划' : '新建计划'} onClose={onClose}>
      <form
        className={styles.form}
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
        <TextField label="标题" value={title} required onChange={(event) => setTitle(event.target.value)} />
        <Textarea label="备注" value={body} onChange={(event) => setBody(event.target.value)} />
        <div className={styles.pair}>
          <Select label="优先级" value={priority} onChange={(event) => setPriority(Number(event.target.value))}>
            {knownPriority ? null : <option value={priority}>{priority}</option>}
            {PRIORITIES.map((item) => (
              <option key={item.value} value={item.value}>
                {item.label}
              </option>
            ))}
          </Select>
          {done ? (
            <TextField id="plan-status" label="状态" value="已完成" disabled />
          ) : (
            <Select
              id="plan-status"
              label="状态"
              value={status}
              onChange={(event) => setStatus(event.target.value as PlanStatus)}
            >
              <option value="inbox">收集箱</option>
              <option value="scheduled">已安排</option>
            </Select>
          )}
        </div>
        {done || status === 'scheduled' ? (
          <div className={styles.pair}>
            <DateField label="安排时间" value={scheduled} onChange={setScheduled} />
            <DateField label="截止时间" value={due} onChange={setDue} />
          </div>
        ) : (
          <DateField label="截止时间" value={due} onChange={setDue} />
        )}
        <div className={styles.steps}>
          <span className={styles.stepsLabel}>步骤</span>
          {steps.map((step) => (
            <div key={step.key} className={styles.step}>
              <Checkbox
                checked={step.done}
                aria-label="步骤完成"
                onChange={(event) =>
                  setSteps((current) =>
                    current.map((item) => (item.key === step.key ? { ...item, done: event.target.checked } : item))
                  )
                }
              />
              <TextField
                className={styles.stepTitle}
                aria-label="步骤标题"
                value={step.title}
                onChange={(event) =>
                  setSteps((current) =>
                    current.map((item) => (item.key === step.key ? { ...item, title: event.target.value } : item))
                  )
                }
              />
              <IconButton
                label="删除步骤"
                onClick={() => setSteps((current) => current.filter((item) => item.key !== step.key))}
              >
                <TrashIcon weight="regular" />
              </IconButton>
            </div>
          ))}
          <Button
            variant="text"
            onClick={() => setSteps((current) => [...current, { key: crypto.randomUUID(), title: '', done: false }])}
          >
            <PlusIcon weight="regular" />
            添加步骤
          </Button>
        </div>
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
      </form>
    </Modal>
  );
}
