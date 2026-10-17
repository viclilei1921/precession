export const PATH = {
  /* 根路径，进入后转到今天 */
  root: '/',
  /* 今天 */
  today: '/today',
  /* 计划 */
  plan: '/plan',
  /* 手记 */
  journal: '/journal',
  /* 手记编辑 */
  journalEntry: '/journal/$entryId',
  /* 成长 */
  growth: '/growth',
  /* 书库 */
  library: '/library',
  /* 阅读器 */
  bookReader: '/library/$bookId',
  /* 工具箱入口，转到视频处理 */
  toolbox: '/toolbox',
  /* 文件加密 */
  toolboxFile: '/toolbox/file',
  /* 图片转码 */
  toolboxImage: '/toolbox/image',
  /* 视频处理 */
  toolboxVideo: '/toolbox/video',
  /* 回顾 */
  review: '/review',
  /* 年度之书 */
  yearBook: '/review/year',
  /* 设置 */
  settings: '/settings'
} as const;
