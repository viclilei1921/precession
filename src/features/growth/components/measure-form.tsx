import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { GrowthMeasure, GrowthMeasureInput } from '@/bridge/growth';
import { growthMeasureCreate, growthMeasureDelete, growthMeasureUpdate } from '@/bridge/growth';
import { Button } from '@/components/button';
import { DateField } from '@/components/date-field';
import { Modal, ModalCloseRef } from '@/components/modal';
import { TextField } from '@/components/text-field';
import { useBackLayer } from '@/layout/mobile/back';
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
  const { t } = useTranslation('growth');
  const queryClient = useQueryClient();
  const closeRef = useRef(onClose);
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
        throw new Error(t('measure.heightWeightError'));
      }
      return measure ? growthMeasureUpdate(measure.id, next) : growthMeasureCreate(next);
    },
    onSuccess: async () => {
      await refreshGrowth(queryClient);
      closeRef.current();
    }
  });
  const remove = useMutation({
    mutationFn: () => {
      if (!measure) {
        return Promise.reject(new Error(t('measure.notFound')));
      }
      return growthMeasureDelete(measure.id);
    },
    onSuccess: async () => {
      await refreshGrowth(queryClient);
      closeRef.current();
    }
  });
  const busy = save.isPending || remove.isPending;
  useBackLayer(true, onClose);
  const error =
    formError || (save.error ? errorMessage(save.error) : '') || (remove.error ? errorMessage(remove.error) : '');

  return (
    <Modal title={measure ? t('measure.editTitle') : t('measure.createTitle')} onClose={onClose}>
      <ModalCloseRef target={closeRef} />
      <form
        className={styles.stack}
        onSubmit={(event) => {
          event.preventDefault();
          if (!input()) {
            setFormError(t('measure.heightWeightError'));
            return;
          }
          setFormError('');
          save.mutate();
        }}
      >
        <DateField label={t('form.date')} value={occurredAt} onChange={setOccurredAt} />
        <div className={styles.pair}>
          <TextField
            label={t('measure.heightLabel')}
            value={height}
            required
            inputMode="decimal"
            onChange={(event) => setHeight(event.target.value)}
          />
          <TextField
            label={t('measure.weightLabel')}
            value={weight}
            required
            inputMode="decimal"
            onChange={(event) => setWeight(event.target.value)}
          />
        </div>
        {error ? <p className={styles.error}>{error}</p> : null}
        {confirmDelete ? <p className={styles.hint}>{t('measure.confirmDelete')}</p> : null}
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
              {confirmDelete ? t('form.confirmDelete') : t('common:button.delete')}
            </Button>
          ) : null}
          <span className={styles.spacer} />
          <Button variant="outlined" disabled={busy} onClick={() => closeRef.current()}>
            {t('common:button.cancel')}
          </Button>
          <Button type="submit" disabled={busy}>
            {t('common:button.save')}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
