import type { JournalEntry, JournalKind } from '@/bridge/journal';
import { JournalEditor } from '@/features/journal/components/editor';

type MobileJournalEditorProps = {
  kind: JournalKind;
  onClose: () => void;
  onSaved: (entry: JournalEntry) => void;
};

/** 移动端写手记。正文和保存仍走桌面编辑器，开关留在地址栏的 create。 */
export function MobileJournalEditor({ kind, onClose, onSaved }: MobileJournalEditorProps) {
  return <JournalEditor entry={null} kind={kind} onClose={onClose} onSaved={onSaved} />;
}
