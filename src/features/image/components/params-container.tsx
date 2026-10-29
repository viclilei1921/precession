import { Checkbox } from '@/components/checkbox';
import { TextField } from '@/components/text-field';
import type { ImageParamsDraft } from '../params-draft';
import styles from './params.module.css';

type ParamsContainerProps = {
  draft: ImageParamsDraft;
  onChange: (draft: ImageParamsDraft) => void;
};

export function ParamsContainer({ draft, onChange }: ParamsContainerProps) {
  function patch(next: Partial<ImageParamsDraft>) {
    onChange({ ...draft, ...next });
  }

  return (
    <div className={styles.row}>
      <TextField
        className={styles.num}
        label="线程数"
        value={draft.jxlThreads}
        inputMode="numeric"
        placeholder="留空"
        onChange={(event) => patch({ jxlThreads: event.target.value })}
      />
      <Checkbox
        label="使用容器"
        checked={draft.useContainer}
        onChange={(event) => patch({ useContainer: event.target.checked })}
      />
      <Checkbox
        label="渐进"
        checked={draft.progressive}
        onChange={(event) => patch({ progressive: event.target.checked })}
      />
    </div>
  );
}
