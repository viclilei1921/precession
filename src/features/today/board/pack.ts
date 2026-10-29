/** 卡片间距，与页面间距档一致 */
export const GAP = 16;
/** 月历组最小宽度 */
export const CAL_MIN = 420;
/** 年历组最小宽度 */
export const YEAR_MIN = 1200;
/** 模块卡最小宽度 */
export const MOD_MIN = 280;

/** 再窄时，日期、休班和农历会挤在一格里 */
const MONTH_MIN = 280;
const MONTH_GAP = 16;
/** 日历卡片左右内边距，年历列数按内容宽度算 */
const CARD_PAD = 32;

export type Box = {
  left: number;
  top: number;
  width: number;
  height: number;
};

export type ColumnSpec = {
  sideBySide: boolean;
  calendarWidth: number;
  columns: { left: number; width: number }[];
};

/** 年历组实际会分到的宽度。并排时按最小宽度的权重分剩余空间。 */
export function groupWidthFor(boardWidth: number, calMin: number): number {
  if (boardWidth <= 0) {
    return 0;
  }
  if (boardWidth < calMin + GAP + MOD_MIN) {
    return boardWidth;
  }
  const moduleCols = Math.max(1, Math.floor((boardWidth - calMin) / (MOD_MIN + GAP)));
  const gaps = moduleCols * GAP;
  const extra = boardWidth - calMin - moduleCols * MOD_MIN - gaps;
  const weight = calMin + moduleCols * MOD_MIN;
  let calendarWidth = calMin + Math.floor((extra * calMin) / weight);
  const moduleWidth = MOD_MIN + Math.floor((extra * MOD_MIN) / weight);
  calendarWidth += boardWidth - (calendarWidth + moduleWidth * moduleCols + gaps);
  return calendarWidth;
}

/** 一行能排几个月。按日历组将得到的宽度算，不读动画中的窄宽度。 */
export function yearColumns(calendarWidth: number): number {
  const inner = Math.max(0, calendarWidth - CARD_PAD);
  return Math.max(1, Math.min(12, Math.floor((inner + MONTH_GAP) / (MONTH_MIN + MONTH_GAP))));
}

/** 日历组和模块列的宽度、横坐标。高度要等测量之后再定。 */
export function columnSpec(boardWidth: number, calMin: number): ColumnSpec {
  if (boardWidth < calMin + GAP + MOD_MIN) {
    const moduleCols = Math.max(1, Math.floor((boardWidth + GAP) / (MOD_MIN + GAP)));
    const base = Math.floor((boardWidth - (moduleCols - 1) * GAP) / moduleCols);
    const leftover = boardWidth - (base * moduleCols + (moduleCols - 1) * GAP);
    const columns: { left: number; width: number }[] = [];
    let cursor = 0;
    for (let index = 0; index < moduleCols; index += 1) {
      const width = base + (index < leftover ? 1 : 0);
      columns.push({ left: cursor, width });
      cursor += width + GAP;
    }
    return { sideBySide: false, calendarWidth: boardWidth, columns };
  }

  const moduleCols = Math.max(1, Math.floor((boardWidth - calMin) / (MOD_MIN + GAP)));
  const gaps = moduleCols * GAP;
  const extra = boardWidth - calMin - moduleCols * MOD_MIN - gaps;
  const weight = calMin + moduleCols * MOD_MIN;
  let calendarWidth = calMin + Math.floor((extra * calMin) / weight);
  const moduleWidth = MOD_MIN + Math.floor((extra * MOD_MIN) / weight);
  calendarWidth += boardWidth - (calendarWidth + moduleWidth * moduleCols + gaps);
  const columns = Array.from({ length: moduleCols }, (_, index) => ({
    left: calendarWidth + GAP + index * (moduleWidth + GAP),
    width: moduleWidth
  }));
  return { sideBySide: true, calendarWidth, columns };
}

/** 当前最矮的那一列 */
export function shortestColumn(tops: number[]): number {
  let column = 0;
  for (let index = 1; index < tops.length; index += 1) {
    if (tops[index] < tops[column]) {
      column = index;
    }
  }
  return column;
}
