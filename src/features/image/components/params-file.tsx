import { Checkbox } from '@/components/checkbox';
import { TextField } from '@/components/text-field';
import type { ImageParamsDraft } from '../params-draft';
import { avifExif, avifIcc, avifJobs, avifProgressive, avifTileCount, avifTiles, avifXmp } from '../params-notes';
import { HelpRow } from './help-row';
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
    <div className={styles.fields}>
      <div className={styles.checks}>
        <HelpRow align="check" label="EXIF 说明" title={avifExif}>
          <Checkbox
            label="忽略 EXIF"
            checked={draft.ignoreExif}
            onChange={(event) => patch({ ignoreExif: event.target.checked })}
          />
        </HelpRow>
        <HelpRow align="check" label="XMP 说明" title={avifXmp}>
          <Checkbox
            label="忽略 XMP"
            checked={draft.ignoreXmp}
            onChange={(event) => patch({ ignoreXmp: event.target.checked })}
          />
        </HelpRow>
        <HelpRow align="check" label="ICC 说明" title={avifIcc}>
          <Checkbox
            label="忽略 ICC"
            checked={draft.ignoreIcc}
            onChange={(event) => patch({ ignoreIcc: event.target.checked })}
          />
        </HelpRow>
        <HelpRow align="check" label="自动分块说明" title={avifTiles}>
          <Checkbox
            label="自动分块"
            checked={draft.autotiling}
            onChange={(event) => patch({ autotiling: event.target.checked })}
          />
        </HelpRow>
        <HelpRow align="check" label="渐进说明" title={avifProgressive}>
          <Checkbox
            label="渐进"
            checked={draft.progressive}
            onChange={(event) => patch({ progressive: event.target.checked })}
          />
        </HelpRow>
      </div>
      {draft.autotiling ? null : (
        <div className={styles.pair}>
          <HelpRow label="分块行说明" title={avifTileCount}>
            <TextField
              label="分块行"
              hint="0 到 6"
              value={draft.tilerowslog2}
              inputMode="numeric"
              onChange={(event) => patch({ tilerowslog2: event.target.value })}
            />
          </HelpRow>
          <HelpRow label="分块列说明" title={avifTileCount}>
            <TextField
              label="分块列"
              hint="0 到 6"
              value={draft.tilecolslog2}
              inputMode="numeric"
              onChange={(event) => patch({ tilecolslog2: event.target.value })}
            />
          </HelpRow>
        </div>
      )}
      <div className={styles.pair}>
        <HelpRow label="线程数说明" title={avifJobs}>
          <TextField
            label="线程数"
            hint="留空则占满核心"
            value={draft.jobs}
            inputMode="numeric"
            onChange={(event) => patch({ jobs: event.target.value })}
          />
        </HelpRow>
      </div>
    </div>
  );
}
