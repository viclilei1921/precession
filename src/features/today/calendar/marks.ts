import type { TimelineItem, TimelineKind } from '@/bridge/timeline';
import { toDateInputValue } from '@/utils/day';

const ORDER = ['plan', 'journal', 'growth', 'library'] as const;

export type MarkTone = (typeof ORDER)[number];

const toneOf: Record<TimelineKind, MarkTone> = {
  plan: 'plan',
  diary: 'journal',
  spark: 'journal',
  writing: 'journal',
  milestone: 'growth',
  moment: 'growth',
  excerpt: 'library',
  note: 'library'
};

/** 每一天有哪些模块留下了记录，同一种只点一个 */
export function marksByDay(items: TimelineItem[]): Map<string, MarkTone[]> {
  const sets = new Map<string, Set<MarkTone>>();
  for (const item of items) {
    const key = toDateInputValue(item.occurredAt);
    const tones = sets.get(key) ?? new Set<MarkTone>();
    tones.add(toneOf[item.kind]);
    sets.set(key, tones);
  }
  const result = new Map<string, MarkTone[]>();
  for (const [key, tones] of sets) {
    result.set(
      key,
      ORDER.filter((tone) => tones.has(tone))
    );
  }
  return result;
}
