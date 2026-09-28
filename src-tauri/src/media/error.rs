use serde::Serialize;

use crate::db::error::DbError;

#[derive(Debug, thiserror::Error)]
pub enum MediaError {
  /// 类型不正确
  #[error("类型不正确")]
  KindInvalid,
  /// 媒体不存在
  #[error("媒体不存在")]
  NotFound,
  /// 所属记录不存在
  #[error("引用不存在")]
  ReferencedMissing,
  /// 数据库未解锁
  #[error("数据库未解锁")]
  Locked,
  /// 操作失败
  #[error("操作失败")]
  Internal,
}

impl From<DbError> for MediaError {
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => MediaError::Locked,
      _ => MediaError::Internal,
    }
  }
}

impl Serialize for MediaError {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
