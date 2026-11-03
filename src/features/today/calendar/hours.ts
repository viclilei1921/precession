import i18n from '@/i18n';

/** 后端用「暂无」表示空值，展示时换成当前语言 */
const NO_DATA = '暂无';

/** 宜忌、神煞等字段是否为空占位 */
export function isNoData(value: string): boolean {
  return value.trim() === NO_DATA;
}

/** 当前语言下的空值文案 */
function noDataLabel(): string {
  return i18n.t('today:hours.none');
}

/** 宜忌词按空白切开。没有内容时按空占位处理。 */
export function adviceWords(value: string): string[] {
  const text = value.trim();
  if (!text) {
    return [];
  }
  if (isNoData(text)) {
    return [noDataLabel()];
  }
  return text.split(/\s+/).filter(Boolean);
}

/** 现在落在哪一个时辰。23 点起算子时。 */
export function currentHourIndex(time = Date.now()): number {
  const hour = new Date(time).getHours();
  if (hour >= 23) {
    return 0;
  }
  return Math.floor((hour + 1) / 2);
}

/** 非今天时，时辰列表对齐辰时（07:00–09:00） */
export const CHEN_HOUR = 4;
