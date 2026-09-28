import { invokeCommand } from './invoke';

/** 月历格子上的农历 */
export type Lunar = {
  year: number;
  month: number;
  day: number;
  isLeap: boolean;
  text: string;
  ganZhi: string;
  animal: string;
  solarTerm: string | null;
};

/** 区间内合成后的一天 */
export type CalendarDay = {
  date: string;
  weekday: number;
  isWeekend: boolean;
  isWorkday: boolean;
  holidayName: string | null;
  officialIsOffDay: boolean | null;
  userOverridden: boolean;
  lunar: Lunar | null;
};

/** 一次区间查询 */
export type CalendarRange = {
  fromDate: string;
  toDate: string;
  days: CalendarDay[];
};

/** 保存某一天是否上班 */
export type CalendarDayInput = {
  isWorkday: boolean;
  note: string | null;
};

/** 个人对某一天的覆盖 */
export type CalendarOverride = {
  id: string;
  date: string;
  isWorkday: boolean;
  note: string | null;
  createdAt: number;
  updatedAt: number;
};

/** 官方节假日里的一天 */
export type OfficialDayInput = {
  name: string;
  date: string;
  isOffDay: boolean;
};

/** 按年替换官方节假日 */
export type OfficialImportInput = {
  year: number;
  days: OfficialDayInput[];
};

/** 官方导入结果 */
export type OfficialImport = {
  year: number;
  count: number;
};

/** 黄历里的农历 */
export type AlmanacLunar = {
  lYear: number;
  lMonth: number;
  lDay: number;
  isLeap: boolean;
  text: string;
};

/** 五行 */
export type AlmanacWuXing = {
  nayin: string;
  ganWx: string;
  zhiWx: string;
};

/** 时辰宜忌 */
export type AlmanacHour = {
  hourIndex: number;
  zhi: string;
  yi: string;
  ji: string;
  timeRange: string;
  chongSha: string;
  caiShen: string;
  xiShen: string;
  fuShen: string;
};

/** 单日黄历 */
export type Almanac = {
  solarDate: string;
  lunar: AlmanacLunar;
  ganZhi: string;
  animal: string;
  ncWeek: string;
  solarTerm: string;
  pengZu: string;
  jiShen: string;
  xiongShen: string;
  jianChu: string;
  daoDay: string;
  taiShen: string;
  yi: string;
  ji: string;
  caiShen: string;
  xiShen: string;
  fuShen: string;
  wuXing: AlmanacWuXing;
  chongSha: string;
  zhiShen: string;
  hourYiJi: AlmanacHour[];
};

/** 列出闭区间内每一天 */
export function calendarList(from: string, to: string) {
  return invokeCommand<CalendarRange>('calendar_list', { from, to });
}

/** 保存某一天是否上班 */
export function calendarDayUpsert(date: string, input: CalendarDayInput) {
  return invokeCommand<CalendarOverride>('calendar_day_upsert', { date, input });
}

/** 去掉某一天的个人安排 */
export function calendarDayDelete(date: string) {
  return invokeCommand<void>('calendar_day_delete', { date });
}

/** 按年替换官方节假日 */
export function calendarOfficialImport(input: OfficialImportInput) {
  return invokeCommand<OfficialImport>('calendar_official_import', { input });
}

/** 读取单日黄历 */
export function calendarAlmanac(date: string) {
  return invokeCommand<Almanac>('calendar_almanac', { date });
}
