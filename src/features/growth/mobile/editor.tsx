import type { GrowthEntry, GrowthKind } from '@/bridge/growth';
import { GrowthEditor } from '@/features/growth/components/editor';

type MobileGrowthEditorProps = {
  ownerId: string;
  kind: GrowthKind;
  onClose: () => void;
  onSaved: (entry: GrowthEntry) => void;
};

/** 移动端记一笔。字段和保存仍走桌面编辑器。 */
export function MobileGrowthEditor({ ownerId, kind, onClose, onSaved }: MobileGrowthEditorProps) {
  return <GrowthEditor entry={null} ownerId={ownerId} kind={kind} onClose={onClose} onSaved={onSaved} />;
}
