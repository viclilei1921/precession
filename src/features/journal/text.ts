import { markdownPlain } from '@/components/markdown';
import i18n from '@/i18n';
import { formatDate, formatMonth, weekdayShort } from '@/utils/day';

type Named = {
  id: string;
  name: string;
};

/** 卡片左侧的日期，和阅读页上的完整日期 */
export function journalDate(time: number) {
  const date = new Date(time);
  return {
    day: String(date.getDate()),
    month: formatMonth(time),
    week: weekdayShort(time),
    full: formatDate(time)
  };
}

/** 卡片上的正文摘要。封存的正文不解开 */
export function journalExcerpt(body: string, locked: boolean) {
  if (locked) {
    return i18n.t('journal:text.sealed');
  }
  return markdownPlain(body) || i18n.t('journal:text.empty');
}

/** 去掉标记后的字数。封存时不计算 */
export function journalWords(body: string, locked: boolean) {
  if (locked) {
    return null;
  }
  return markdownPlain(body).replace(/\s/g, '').length;
}

/** 按 id 取出还在的名字，保持原来的顺序 */
export function lookupNames(ids: string[], items: Named[]) {
  const byId = new Map(items.map((item) => [item.id, item.name]));
  return ids.flatMap((id) => {
    const name = byId.get(id);
    return name ? [name] : [];
  });
}
