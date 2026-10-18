import { Button } from '@/components/button';
import { Card } from '@/components/card';
import { TimeField } from '@/components/time-field';
import styles from './clip-fields.module.css';

type ClipFieldsProps = {
  index: number;
  start: number;
  end: number;
  canRemove: boolean;
  onStart: (start: number) => void;
  onEnd: (end: number) => void;
  onRemove: () => void;
};

/** 一段裁剪的开始和结束。 */
export function ClipFields({ index, start, end, canRemove, onStart, onEnd, onRemove }: ClipFieldsProps) {
  return (
    <Card title={`片段 ${index + 1}`}>
      <div className={styles.pair}>
        <TimeField label="开始" value={start} onChange={onStart} />
        <TimeField label="结束" value={end} error={end <= start ? '结束要晚于开始' : undefined} onChange={onEnd} />
      </div>
      {canRemove ? (
        <Button variant="text" onClick={onRemove}>
          删除这段
        </Button>
      ) : null}
    </Card>
  );
}
