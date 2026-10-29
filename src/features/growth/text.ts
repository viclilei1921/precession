import type { GrowthEntry } from '@/bridge/growth';
import { ageLabel } from '@/utils/day';

const CHILD_MONTHS = 18 * 12;

export type GrowthChapter = {
  key: string;
  title: string;
  subtitle: string;
  entries: GrowthEntry[];
};

/** 从生日到某一天相差的整月，未出生为负数 */
function ageMonths(birthday: number, at: number): number {
  const birth = new Date(birthday);
  const when = new Date(at);
  let months = (when.getFullYear() - birth.getFullYear()) * 12 + (when.getMonth() - birth.getMonth());
  if (when.getDate() < birth.getDate()) {
    months -= 1;
  }
  return months;
}

/** 有生日且现在不满 18 岁，轨迹按周岁分章 */
export function isYoung(birthday: number | null, at = Date.now()): boolean {
  if (birthday == null) {
    return false;
  }
  const months = ageMonths(birthday, at);
  return months >= 0 && months < CHILD_MONTHS;
}

/** 卡片上的当时年龄。出生当天单独写，小孩子前面加「当时」 */
export function cardAge(birthday: number | null, at: number, young: boolean): string {
  if (birthday == null) {
    return '';
  }
  const label = ageLabel(birthday, at);
  if (label === '') {
    return '';
  }
  if (label === '0 个月') {
    return '出生当天';
  }
  return young ? `当时 ${label}` : label;
}

/** 章节副标题。0 个月写成出生当天 */
function chapterSubtitle(birthday: number | null, at: number): string {
  if (birthday == null) {
    return '';
  }
  const label = ageLabel(birthday, at);
  if (label === '' || label === '0 个月') {
    return label === '0 个月' ? '出生当天' : '';
  }
  return label;
}

function chapterOf(birthday: number | null, occurredAt: number, young: boolean) {
  if (!young || birthday == null) {
    const year = new Date(occurredAt).getFullYear();
    return { key: `year-${year}`, title: String(year) };
  }
  const years = Math.floor(Math.max(0, ageMonths(birthday, occurredAt)) / 12);
  if (years <= 0) {
    return { key: 'age-0', title: '不满一岁' };
  }
  return { key: `age-${years}`, title: `${years} 岁` };
}

/** 把已经按时间倒序的记录分成章。小孩子按周岁，其他人按公历年 */
export function groupChapters(entries: GrowthEntry[], birthday: number | null, young: boolean): GrowthChapter[] {
  const chapters: GrowthChapter[] = [];
  for (const entry of entries) {
    const next = chapterOf(birthday, entry.occurredAt, young);
    const current = chapters[chapters.length - 1];
    if (current?.key === next.key) {
      current.entries.push(entry);
    } else {
      chapters.push({ ...next, subtitle: '', entries: [entry] });
    }
  }
  return chapters.map((chapter) => ({
    ...chapter,
    subtitle: chapterSubtitle(birthday, chapter.entries[0]?.occurredAt ?? 0)
  }));
}

/** 毫米换成厘米，保留一位 */
export function cmText(mm: number): string {
  return (mm / 10).toFixed(1);
}

/** 克换成千克，保留一位 */
export function kgText(grams: number): string {
  return (grams / 1000).toFixed(1);
}

/** 厘米输入换成毫米。空的或不是正数时返回 null */
export function parseCm(value: string): number | null {
  const size = Number(value);
  if (!Number.isFinite(size) || size <= 0) {
    return null;
  }
  return Math.round(size * 10);
}

/** 千克输入换成克。空的或不是正数时返回 null */
export function parseKg(value: string): number | null {
  const size = Number(value);
  if (!Number.isFinite(size) || size <= 0) {
    return null;
  }
  return Math.round(size * 1000);
}
