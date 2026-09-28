export const PATH = {
  /* 首页 */
  home: '/',
  /* 今天 */
  today: '/today',
  /* 计划 */
  plan: '/plan',
  /* 手记 */
  journal: '/journal',
  /* 手记编辑 */
  journalEditor: '/journal/$entryId',
  /* 成长 */
  growth: '/growth',
  /* 书库 */
  library: '/library',
  /* 阅读器 */
  reader: '/library/$bookId',
  /* 工具箱 */
  toolbox: '/toolbox',
  /* 回顾 */
  review: '/review',
  /* 年度之书 */
  year: '/review/year',
  /* 设置 */
  settings: '/settings'
} as const;
