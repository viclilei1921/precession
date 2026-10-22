import { Checkbox } from '@/components/checkbox';
import { TextField } from '@/components/text-field';
import type { ImageParamsDraft } from '../params-draft';
import { jxlContainer, jxlProgressive, jxlThreads } from '../params-notes';
import { HelpRow } from './help-row';
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
    <div className={styles.fields}>
      <div className={styles.checks}>
        <HelpRow align="check" label="容器说明" title={jxlContainer}>
          <Checkbox
            label="使用容器"
            checked={draft.useContainer}
            onChange={(event) => patch({ useContainer: event.target.checked })}
          />
        </HelpRow>
        <HelpRow align="check" label="渐进说明" title={jxlProgressive}>
          <Checkbox
            label="渐进"
            checked={draft.progressive}
            onChange={(event) => patch({ progressive: event.target.checked })}
          />
        </HelpRow>
      </div>
      <div className={styles.pair}>
        <HelpRow label="线程数说明" title={jxlThreads}>
          <TextField
            label="线程数"
            hint="留空则用默认"
            value={draft.jxlThreads}
            inputMode="numeric"
            onChange={(event) => patch({ jxlThreads: event.target.value })}
          />
        </HelpRow>
      </div>
    </div>
  );
}
