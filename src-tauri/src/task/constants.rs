/// 任务更新事件
pub const EVENT_TASK: &str = "task-update";
/// 队列更新事件
pub const EVENT_QUEUE: &str = "queue-updated";

/// 任务状态 - 待处理
pub const STATUS_PENDING: &str = "pending";
/// 任务状态 - 处理中
pub const STATUS_PROCESSING: &str = "processing";
/// 任务状态 - 完成
pub const STATUS_COMPLETED: &str = "completed";
/// 任务状态 - 失败
pub const STATUS_FAILED: &str = "failed";
/// 任务状态 - 取消
pub const STATUS_CANCELED: &str = "canceled";

/// 任务类型 - 加密文件
pub const KIND_ENCRYPT_FILE: &str = "encryptFile";
/// 任务类型 - 解密文件
pub const KIND_DECRYPT_FILE: &str = "decryptFile";
/// 任务类型 - 导入媒体
pub const KIND_IMPORT_MEDIA: &str = "importMedia";
/// 任务类型 - 加密媒体
pub const KIND_ENCRYPT_MEDIA: &str = "encryptMedia";
/// 任务类型 - 解密媒体
pub const KIND_DECRYPT_MEDIA: &str = "decryptMedia";
/// 任务类型 - 转换视频
pub const KIND_CONVERT_VIDEO: &str = "convertVideo";
/// 任务类型 - 剪辑视频
pub const KIND_CUT_VIDEO: &str = "cutVideo";
/// 任务类型 - 合并视频
pub const KIND_MERGE_VIDEO: &str = "mergeVideo";
/// 任务类型 - 添加视频
pub const KIND_APPEND_VIDEO: &str = "appendVideo";
/// 任务类型 - 转换 AVIF
pub const KIND_CONVERT_AVIF: &str = "convertAvif";
/// 任务类型 - 转换 JXL
pub const KIND_CONVERT_JXL: &str = "convertJxl";

/// 进度步长
pub(super) const PROGRESS_STEP: u64 = 1024 * 1024;
