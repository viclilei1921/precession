use serde::Serialize;

use crate::db::error::DbError;
use crate::upload::error::UploadError;

/// 任务操作失败
///
/// 序列化时只写出错误文本。数据库锁定变成 `Locked`，其余数据库错误按同名变体转过来。侧车取消变成 `Canceled`，其余侧车错误包进 `Failed`。
///
/// # 变体
/// - `KindInvalid`: 类型不正确
/// - `PathEmpty`: 路径不能为空
/// - `Missing`: 文件不存在
/// - `PasswordEmpty`: 密码不能为空
/// - `SamePath`: 输入和输出不能是同一个文件
/// - `NotFound`: 上传文件不存在
/// - `ReferencedMissing`: 引用不存在
/// - `Locked`: 数据库未解锁
/// - `AlreadyEncrypted`: 文件已经加密
/// - `NotEncrypted`: 文件还没有加密
/// - `Busy`: 这个文件正在处理
/// - `Running`: 正在处理，不能删除
/// - `BadFormat`: 无效的加密格式
/// - `BadVersion`: 不支持的加密版本
/// - `Truncated`: 密文太短
/// - `Canceled`: 已取消
/// - `DesktopOnly`: 仅桌面端支持。非桌面端才有这个变体
/// - `Failed`: 转码或侧车失败，载荷是具体原因
/// - `Internal`: 操作失败
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
  /// 类型不正确
  #[error("类型不正确")]
  KindInvalid,
  /// 路径不能为空
  #[error("路径不能为空")]
  PathEmpty,
  /// 文件不存在
  #[error("文件不存在")]
  Missing,
  /// 密码不能为空
  #[error("密码不能为空")]
  PasswordEmpty,
  /// 输入和输出不能是同一个文件
  #[error("输入和输出不能是同一个文件")]
  SamePath,
  /// 上传文件不存在
  #[error("上传文件不存在")]
  NotFound,
  /// 引用不存在
  #[error("引用不存在")]
  ReferencedMissing,
  /// 数据库未解锁
  #[error("数据库未解锁")]
  Locked,
  /// 文件已经加密
  #[error("文件已经加密")]
  AlreadyEncrypted,
  /// 文件还没有加密
  #[error("文件还没有加密")]
  NotEncrypted,
  /// 这个文件正在处理
  #[error("这个文件正在处理")]
  Busy,
  /// 正在处理，不能删除
  #[error("正在处理，不能删除")]
  Running,
  /// 无效的加密格式
  #[error("无效的加密格式")]
  BadFormat,
  /// 不支持的加密版本
  #[error("不支持的加密版本")]
  BadVersion,
  /// 密文太短
  #[error("密文太短")]
  Truncated,
  /// 已取消
  #[error("已取消")]
  Canceled,
  /// 仅桌面端支持
  #[cfg(not(desktop))]
  #[error("仅桌面端支持")]
  DesktopOnly,
  /// 转码或侧车失败，消息来自具体原因
  #[error("{0}")]
  Failed(String),
  /// 操作失败
  #[error("操作失败")]
  Internal,
}

impl From<DbError> for TaskError {
  /// 把数据库锁定映射为 `Locked`，其余映射为 `Internal`
  ///
  /// # 参数
  /// - `err`: 数据库错误
  ///
  /// # 返回值
  /// 对应的 `TaskError`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let err = TaskError::from(DbError::Locked);
  /// assert!(matches!(err, TaskError::Locked));
  /// ```
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => TaskError::Locked,
      _ => TaskError::Internal,
    }
  }
}

impl From<UploadError> for TaskError {
  /// 按同名变体把上传文件错误映射成任务错误
  ///
  /// # 参数
  /// - `err`: 上传文件错误
  ///
  /// # 返回值
  /// 对应的 `TaskError`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let err = TaskError::from(UploadError::NotFound);
  /// assert!(matches!(err, TaskError::NotFound));
  /// ```
  fn from(err: UploadError) -> Self {
    match err {
      UploadError::KindInvalid => TaskError::KindInvalid,
      UploadError::NotFound => TaskError::NotFound,
      UploadError::ReferencedMissing => TaskError::ReferencedMissing,
      UploadError::Locked => TaskError::Locked,
      UploadError::Encrypted => TaskError::AlreadyEncrypted,
      UploadError::NeedPassword => TaskError::PasswordEmpty,
      UploadError::Password => TaskError::Failed("密码不正确".into()),
      UploadError::Format => TaskError::BadFormat,
      UploadError::OwnerSealed => TaskError::Failed("记录已封存".into()),
      UploadError::TooLarge => TaskError::Internal,
      UploadError::Internal => TaskError::Internal,
    }
  }
}

impl Serialize for TaskError {
  /// 把错误的显示文本序列化成字符串
  ///
  /// # 参数
  /// - `serializer`: serde 序列化器
  ///
  /// # 返回值
  /// 序列化成功时的输出
  ///
  /// # 错误
  /// `serialize_str` 失败时返回 `S::Error`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let text = serde_json::to_string(&TaskError::Canceled).unwrap();
  /// assert!(text.contains("已取消"));
  /// ```
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}

#[cfg(desktop)]
impl From<crate::sidecar::SidecarError> for TaskError {
  /// 把取消保持为 `Canceled`，其余包进 `Failed`
  ///
  /// # 参数
  /// - `err`: 侧车错误
  ///
  /// # 返回值
  /// 对应的 `TaskError`。非取消时消息是侧车错误的显示文本
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let err = TaskError::from(crate::sidecar::SidecarError::canceled());
  /// assert!(matches!(err, TaskError::Canceled));
  /// ```
  fn from(err: crate::sidecar::SidecarError) -> Self {
    match err {
      crate::sidecar::SidecarError::Canceled => TaskError::Canceled,
      other => TaskError::Failed(other.to_string()),
    }
  }
}
