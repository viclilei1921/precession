import type { GrowthEntry } from '@/bridge/growth';
import i18n from '@/i18n';
import { ageLabel, ageParts } from '@/utils/day';

const CHILD_MONTHS = 18 * 12;

export type GrowthChapter = {
  key: string;
  title: string;
  subtitle: string;
  entries: GrowthEntry[];
};

/** 有生日且现在不满 18 岁，轨迹按周岁分章 */
export function isYoung(birthday: number | null, at = Date.now()): boolean {
  if (birthday == null) {
    return false;
  }
  const { months } = ageParts(birthday, at);
  return months >= 0 && months < CHILD_MONTHS;
}

/** 卡片上的当时年龄。出生当天单独写，小孩子前面加「当时」 */
export function cardAge(birthday: number | null, at: number, young: boolean): string {
  if (birthday == null) {
    return '';
  }
  const { months } = ageParts(birthday, at);
  if (months < 0) {
    return '';
  }
  if (months === 0) {
    return i18n.t('growth:text.birthDay');
  }
  const label = ageLabel(birthday, at);
  return young ? i18n.t('growth:text.thenAge', { label }) : label;
}

/** 章节副标题。0 个月写成出生当天 */
function chapterSubtitle(birthday: number | null, at: number): string {
  if (birthday == null) {
    return '';
  }
  const { months } = ageParts(birthday, at);
  if (months < 0) {
    return '';
  }
  if (months === 0) {
    return i18n.t('growth:text.birthDay');
  }
  return ageLabel(birthday, at);
}

function chapterOf(birthday: number | null, occurredAt: number, young: boolean) {
  if (!young || birthday == null) {
    const year = new Date(occurredAt).getFullYear();
    return { key: `year-${year}`, title: String(year) };
  }
  const years = Math.floor(Math.max(0, ageParts(birthday, occurredAt).months) / 12);
  if (years <= 0) {
    return { key: 'age-0', title: i18n.t('growth:text.age0') };
  }
  return { key: `age-${years}`, title: i18n.t('growth:text.ageYears', { count: years, years }) };
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
