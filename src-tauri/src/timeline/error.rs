use serde::Serialize;

use crate::db::error::DbError;

#[derive(Debug, thiserror::Error)]
pub enum TimelineError {
  /// 类型不正确
  #[error("类型不正确")]
  KindInvalid,
  /// 数据库未解锁
  #[error("数据库未解锁")]
  Locked,
  /// 操作失败
  #[error("操作失败")]
  Internal,
}

impl From<DbError> for TimelineError {
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => TimelineError::Locked,
      _ => TimelineError::Internal,
    }
  }
}

impl Serialize for TimelineError {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
