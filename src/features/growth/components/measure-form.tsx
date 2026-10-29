import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import type { GrowthMeasure, GrowthMeasureInput } from '@/bridge/growth';
import { growthMeasureCreate, growthMeasureDelete, growthMeasureUpdate } from '@/bridge/growth';
import { Button } from '@/components/button';
import { DateField } from '@/components/date-field';
import { Modal } from '@/components/modal';
import { TextField } from '@/components/text-field';
import { refreshGrowth } from '@/query/growth/query';
import { fromDateInputValue, startOfLocalDay, toDateInputValue } from '@/utils/day';
import { errorMessage } from '@/utils/error';
import styles from '../page.module.css';
import { cmText, kgText, parseCm, parseKg } from '../text';

type MeasureFormProps = {
  memberId: string;
  measure: GrowthMeasure | null;
  onClose: () => void;
};

/** 记一次身高体重。界面用厘米和千克，各保留一位。 */
export function MeasureForm({ memberId, measure, onClose }: MeasureFormProps) {
  const queryClient = useQueryClient();
  const [occurredAt, setOccurredAt] = useState(toDateInputValue(measure?.occurredAt ?? startOfLocalDay()));
  const [height, setHeight] = useState(measure ? cmText(measure.heightMm) : '');
  const [weight, setWeight] = useState(measure ? kgText(measure.weightG) : '');
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [formError, setFormError] = useState('');

  function input(): GrowthMeasureInput | null {
    const heightMm = parseCm(height);
    const weightG = parseKg(weight);
    const time = fromDateInputValue(occurredAt);
    if (heightMm == null || weightG == null || time == null) {
      return null;
    }
    return { memberId, occurredAt: time, heightMm, weightG };
  }

  const save = useMutation({
    mutationFn: async () => {
      const next = input();
      if (!next) {
        throw new Error('身高和体重要大于 0');
      }
      return measure ? growthMeasureUpdate(measure.id, next) : growthMeasureCreate(next);
    },
    onSuccess: async () => {
      await refreshGrowth(queryClient);
      onClose();
    }
  });
  const remove = useMutation({
    mutationFn: () => {
      if (!measure) {
        return Promise.reject(new Error('记录不存在'));
      }
      return growthMeasureDelete(measure.id);
    },
    onSuccess: async () => {
      await refreshGrowth(queryClient);
      onClose();
    }
  });
  const busy = save.isPending || remove.isPending;
  const error =
    formError || (save.error ? errorMessage(save.error) : '') || (remove.error ? errorMessage(remove.error) : '');

  return (
    <Modal title={measure ? '修改生长' : '记一次'} onClose={onClose}>
      <form
        className={styles.stack}
        onSubmit={(event) => {
          event.preventDefault();
          if (!input()) {
            setFormError('身高和体重要大于 0');
            return;
          }
          setFormError('');
          save.mutate();
        }}
      >
        <DateField label="日期" value={occurredAt} onChange={setOccurredAt} />
        <div className={styles.pair}>
          <TextField
            label="身高（厘米）"
            value={height}
            required
            inputMode="decimal"
            onChange={(event) => setHeight(event.target.value)}
          />
          <TextField
            label="体重（千克）"
            value={weight}
            required
            inputMode="decimal"
            onChange={(event) => setWeight(event.target.value)}
          />
        </div>
        {error ? <p className={styles.error}>{error}</p> : null}
        {confirmDelete ? <p className={styles.hint}>再点一次就会删掉这次记录。</p> : null}
        <div className={styles.actions}>
          {measure ? (
            <Button
              variant="outlined"
              tone="destructive"
              disabled={busy}
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
          <Button variant="outlined" disabled={busy} onClick={onClose}>
            取消
          </Button>
          <Button type="submit" disabled={busy}>
            保存
          </Button>
        </div>
      </form>
    </Modal>
  );
}
