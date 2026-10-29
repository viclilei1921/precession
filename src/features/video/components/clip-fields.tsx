import { TrashIcon } from '@phosphor-icons/react';
import { IconButton } from '@/components/button/icon';
import { TimeField } from '@/components/time-field';
import styles from './clip-fields.module.css';

type ClipFieldsProps = {
  start: number;
  end: number;
  canRemove: boolean;
  onStart: (start: number) => void;
  onEnd: (end: number) => void;
  onRemove: () => void;
};

/** 一段裁剪：开始、结束，排得下就和别的段并排。 */
export function ClipFields({ start, end, canRemove, onStart, onEnd, onRemove }: ClipFieldsProps) {
  const invalid = end <= start;
  return (
    <div className={styles.clip} title={invalid ? '结束要晚于开始' : undefined}>
      <TimeField className={styles.time} ariaLabel="开始" value={start} onChange={onStart} />
      <span className={styles.dash} aria-hidden="true">
        -
      </span>
      <TimeField className={styles.time} ariaLabel="结束" value={end} invalid={invalid} onChange={onEnd} />
      {canRemove ? (
        <IconButton label="删除这段" tone="destructive" size="compact" onClick={onRemove}>
          <TrashIcon weight="regular" />
        </IconButton>
      ) : null}
    </div>
  );
}
