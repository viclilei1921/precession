import type { Plan, PlanGroup } from '@/bridge/plan';
import { PlanDrawer } from '@/features/plan/components/plan-drawer';
import type { PlanPreset } from '@/features/plan/layout';
import { Sheet } from '@/layout/mobile/sheet';
import styles from './detail-sheet.module.css';

type PlanDetailSheetProps = {
  plan: Plan | null;
  preset: PlanPreset;
  groups: PlanGroup[];
  onClose: () => void;
  onSaved: (start: number | null) => void;
};

/** 计划详情。表单仍用桌面那份，外面换成底部面板。 */
export function PlanDetailSheet({ plan, preset, groups, onClose, onSaved }: PlanDetailSheetProps) {
  return (
    <Sheet tall title={plan ? '计划详情' : '新建计划'} onClose={onClose}>
      <div className={styles.host}>
        <PlanDrawer
          key={plan?.id ?? `${preset.date}-${preset.time}-${preset.timeEnd}`}
          plan={plan}
          preset={preset}
          groups={groups}
          onClose={onClose}
          onSaved={onSaved}
        />
      </div>
    </Sheet>
  );
}
