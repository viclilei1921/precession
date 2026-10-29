import { Checkbox } from '@/components/checkbox';
import { TextField } from '@/components/text-field';
import type { ImageParamsDraft } from '../params-draft';
import styles from './params.module.css';

type ParamsFileProps = {
  draft: ImageParamsDraft;
  onChange: (draft: ImageParamsDraft) => void;
};

export function ParamsFile({ draft, onChange }: ParamsFileProps) {
  function patch(next: Partial<ImageParamsDraft>) {
    onChange({ ...draft, ...next });
  }

  return (
    <div className={styles.row}>
      <TextField
        className={styles.num}
        label="线程数"
        value={draft.jobs}
        inputMode="numeric"
        placeholder="留空"
        onChange={(event) => patch({ jobs: event.target.value })}
      />
      {draft.autotiling ? null : (
        <>
          <TextField
            className={styles.num}
            label="分块行"
            value={draft.tilerowslog2}
            inputMode="numeric"
            placeholder="0–6"
            onChange={(event) => patch({ tilerowslog2: event.target.value })}
          />
          <TextField
            className={styles.num}
            label="分块列"
            value={draft.tilecolslog2}
            inputMode="numeric"
            placeholder="0–6"
            onChange={(event) => patch({ tilecolslog2: event.target.value })}
          />
        </>
      )}
      <Checkbox
        label="忽略 EXIF"
        checked={draft.ignoreExif}
        onChange={(event) => patch({ ignoreExif: event.target.checked })}
      />
      <Checkbox
        label="忽略 XMP"
        checked={draft.ignoreXmp}
        onChange={(event) => patch({ ignoreXmp: event.target.checked })}
      />
      <Checkbox
        label="忽略 ICC"
        checked={draft.ignoreIcc}
        onChange={(event) => patch({ ignoreIcc: event.target.checked })}
      />
      <Checkbox
        label="自动分块"
        checked={draft.autotiling}
        onChange={(event) => patch({ autotiling: event.target.checked })}
      />
      <Checkbox
        label="渐进"
        checked={draft.progressive}
        onChange={(event) => patch({ progressive: event.target.checked })}
      />
    </div>
  );
}
