import i18n from '@/i18n';

/**
 * 后端和前端抛出的中文错误文本 → `errors` 命名空间里的 key。
 *
 * Rust 侧把错误序列化成 `#[error("...")]` 的原文，前端少数兜底路径也掷中文。
 * 命中时按当前语言展示，没有命中时保留原文，避免丢失后端附带的细节。
 */
const MESSAGE_KEYS: Record<string, string> = {
  // 通用 / 数据库
  操作失败: 'errors:generic.actionFailed',
  读取失败: 'errors:generic.readFailed',
  已取消: 'errors:generic.cancelled',
  标题不能为空: 'errors:generic.titleRequired',
  名称不能为空: 'errors:generic.nameRequired',
  名称已被使用: 'errors:generic.nameTaken',
  类型不正确: 'errors:generic.kindInvalid',
  状态不正确: 'errors:generic.statusInvalid',
  引用不存在: 'errors:generic.referenceMissing',
  记录不存在: 'errors:generic.notFound',
  标签不存在: 'errors:generic.tagNotFound',
  仅桌面端支持: 'errors:generic.desktopOnly',
  数据库未解锁: 'errors:generic.dbLocked',
  数据库不存在: 'errors:generic.dbNotFound',
  数据库已存在: 'errors:generic.dbExists',
  数据库损坏: 'errors:generic.dbCorrupt',
  数据库已解锁: 'errors:generic.dbUnlocked',
  无法访问数据目录: 'errors:generic.dataDirUnavailable',

  // 密码 / 设备解锁
  密码不能为空: 'errors:auth.passwordEmpty',
  密码不正确: 'errors:auth.passwordWrong',
  两次输入的密码不一致: 'errors:auth.passwordMismatch',
  '密码至少 8 位': 'errors:auth.passwordTooShort',
  请输入密码: 'errors:auth.passwordRequired',
  密钥材料损坏: 'errors:auth.keyCorrupt',
  加密操作失败: 'errors:auth.cryptoFailed',
  这台设备不支持设备解锁: 'errors:auth.deviceUnsupported',
  已取消设备验证: 'errors:auth.deviceCancelled',
  设备解锁已失效: 'errors:auth.deviceExpired',

  // 档案
  没有选择位置: 'errors:archive.noLocation',
  请选数据目录以外的位置: 'errors:archive.outsideDataDir',
  '还有任务在处理，完成后再试': 'errors:archive.busy',
  这不是可以导入的档案: 'errors:archive.invalidArchive',
  无法读写档案: 'errors:archive.ioFailed',
  无法打开系统分享: 'errors:archive.shareUnavailable',

  // 日历
  日期格式不正确: 'errors:calendar.invalidDate',
  结束日期不能早于开始日期: 'errors:calendar.endBeforeStart',
  '日期区间不能超过 366 天': 'errors:calendar.rangeTooLong',
  '日期年份必须在 1900 到 2100 之间': 'errors:calendar.dateYearOutOfRange',
  '年份必须在 1970 到 2100 之间': 'errors:calendar.yearOutOfRange',
  官方日期不能为空: 'errors:calendar.officialDateRequired',
  节日名不能为空: 'errors:calendar.holidayNameRequired',
  日期必须属于该年: 'errors:calendar.dateNotInYear',
  日期重复: 'errors:calendar.duplicateDate',
  这一天没有个人安排: 'errors:calendar.noSchedule',

  // 计划
  步骤标题不能为空: 'errors:plan.stepTitleRequired',
  系统清单不能删除: 'errors:plan.systemListProtected',
  父计划不正确: 'errors:plan.parentInvalid',
  重复设置不正确: 'errors:plan.repeatInvalid',
  评论不能为空: 'errors:plan.commentRequired',
  计划不存在: 'errors:plan.notFound',

  // 手记
  手记不存在: 'errors:journal.notFound',
  正文无法读取: 'errors:journal.bodyUnreadable',

  // 成长
  成长记录需要指定成员: 'errors:growth.memberRequired',
  '身高和体重要大于 0': 'errors:growth.measureInvalid',
  成长记录不存在: 'errors:growth.notFound',
  成员不存在: 'errors:growth.memberNotFound',
  地点不存在: 'errors:growth.placeNotFound',

  // 书库
  进度不正确: 'errors:library.progressInvalid',
  不支持的电子书格式: 'errors:library.unsupportedFormat',
  书籍文件不存在: 'errors:library.bookFileMissing',
  章节索引越界: 'errors:library.chapterOutOfRange',
  无法解析该电子书: 'errors:library.parseFailed',
  这本书还没有正文: 'errors:library.bookEmpty',
  '这本书已加密，暂时不能阅读': 'errors:library.bookEncrypted',

  // 文件
  文件不存在: 'errors:files.notFound',
  无法读取: 'errors:files.unreadable',
  文件记录不存在: 'errors:files.recordNotFound',
  路径不能为空: 'errors:files.pathRequired',
  输入和输出不能是同一个文件: 'errors:files.sameFile',
  上传文件不存在: 'errors:files.uploadMissing',
  文件已经加密: 'errors:files.alreadyEncrypted',
  文件还没有加密: 'errors:files.notEncrypted',
  记录已封存: 'errors:files.sealed',
  文件太大: 'errors:files.tooLarge',
  这个文件正在处理: 'errors:files.processing',
  '正在处理，不能删除': 'errors:files.processingCannotDelete',
  导入失败: 'errors:files.importFailed',
  导入没有返回文件: 'errors:files.importNoOutput',
  不支持的文件格式: 'errors:files.unsupportedFormat',

  // 媒体 / 加密
  无效的加密格式: 'errors:media.invalidCipher',
  不支持的加密版本: 'errors:media.unsupportedCipherVersion',
  密文太短: 'errors:media.cipherTooShort',
  '请先添加图片。': 'errors:media.addImageFirst',
  '请先选择图片。': 'errors:media.selectImageFirst',
  '先保存，再插入图片': 'errors:media.saveBeforeInsert',
  '视频太短，无法预估': 'errors:media.previewTooShort',
  无法读取视频画面: 'errors:media.previewNoPicture',
  '当前 FFmpeg 没有画质滤镜，需要按文档重新编译侧车': 'errors:media.previewNoFilters',
  '当前 FFmpeg 不能做画质预估，需要按文档重新编译侧车': 'errors:media.previewSidecar',
  '找不到 VMAF 模型，需要更新应用资源': 'errors:media.previewNoModel'
};

/** 侧车启动失败：`{name} 启动失败: {detail}` */
const SIDECAR_START_FAILED = /^(.+?)\s启动失败[:：]\s*([\s\S]*)$/;

function rawMessage(error: unknown): string {
  if (typeof error === 'string') {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  if (error && typeof error === 'object' && 'message' in error) {
    const message = (error as { message?: unknown }).message;
    if (typeof message === 'string') {
      return message;
    }
  }
  return '';
}

/** 取出未经翻译的原始错误文本，用于按稳定错误类型分支。 */
export function rawErrorMessage(error: unknown): string {
  return rawMessage(error);
}

/**
 * 取出适合展示的错误文本。
 *
 * 已知的中文错误按 `errors` 命名空间翻译；未知错误原样返回；
 * 完全取不到信息时给一句通用的失败文案。
 */
export function errorMessage(error: unknown): string {
  const raw = rawMessage(error);
  const text = raw.trim();
  if (!text) {
    return i18n.t('errors:generic.unknown');
  }
  const key = MESSAGE_KEYS[text];
  if (key) {
    return i18n.t(key);
  }
  const sidecar = SIDECAR_START_FAILED.exec(text);
  if (sidecar) {
    return i18n.t('errors:media.sidecarStartFailed', { name: sidecar[1], detail: sidecar[2] });
  }
  return raw;
}
