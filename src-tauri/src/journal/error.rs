use serde::Serialize;

use crate::db::error::DbError;

#[derive(Debug, thiserror::Error)]
pub enum JournalError {
  /// 标题不能为空
  #[error("标题不能为空")]
  TitleEmpty,
  /// 类型不正确
  #[error("类型不正确")]
  KindInvalid,
  /// 手记不存在
  #[error("手记不存在")]
  NotFound,
  /// 引用不存在
  #[error("引用不存在")]
  ReferencedMissing,
  /// 数据库未解锁
  #[error("数据库未解锁")]
  Locked,
  /// 操作失败
  #[error("操作失败")]
  Internal,
}

impl From<DbError> for JournalError {
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => JournalError::Locked,
      _ => JournalError::Internal,
    }
  }
}

impl Serialize for JournalError {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
