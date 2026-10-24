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
  /* 书页 */
  bookReader: '/library/$bookId',
  /* 正文阅读 */
  bookRead: '/library/$bookId/read',
  /* 视频处理 */
  video: '/video',
  /* 图片转码 */
  image: '/image',
  /* 文件加密 */
  cipher: '/cipher',
  /* 旧工具箱地址，转到视频处理 */
  toolbox: '/toolbox',
  /* 旧文件加密地址 */
  toolboxFile: '/toolbox/file',
  /* 旧图片转码地址 */
  toolboxImage: '/toolbox/image',
  /* 旧视频处理地址 */
  toolboxVideo: '/toolbox/video',
  /* 回顾 */
  review: '/review',
  /* 年度之书 */
  yearBook: '/review/year',
  /* 设置 */
  settings: '/settings'
} as const;
