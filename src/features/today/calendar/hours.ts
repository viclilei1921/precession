/** 宜忌词按空白切开。没有内容时保持「暂无」。 */
export function adviceWords(value: string): string[] {
  const text = value.trim();
  if (!text || text === '暂无') {
    return text ? [text] : [];
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
