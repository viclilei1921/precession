use serde::Serialize;

use crate::db::error::DbError;
use crate::media::error::MediaError;

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
  /// 媒体不存在
  #[error("媒体不存在")]
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
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => TaskError::Locked,
      _ => TaskError::Internal,
    }
  }
}

impl From<MediaError> for TaskError {
  fn from(err: MediaError) -> Self {
    match err {
      MediaError::KindInvalid => TaskError::KindInvalid,
      MediaError::NotFound => TaskError::NotFound,
      MediaError::ReferencedMissing => TaskError::ReferencedMissing,
      MediaError::Locked => TaskError::Locked,
      MediaError::Internal => TaskError::Internal,
    }
  }
}

impl Serialize for TaskError {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}

#[cfg(desktop)]
impl From<crate::sidecar::SidecarError> for TaskError {
  fn from(err: crate::sidecar::SidecarError) -> Self {
    match err {
      crate::sidecar::SidecarError::Canceled => TaskError::Canceled,
      other => TaskError::Failed(other.to_string()),
    }
  }
}
