use serde::Serialize;

use crate::db::error::DbError;

#[derive(Debug, thiserror::Error)]
pub enum DemoError {
  /// 标题不能为空
  #[error("标题不能为空")]
  TitleEmpty,
  /// 记录不存在
  #[error("记录不存在")]
  RecordNotFound,
  /// 数据库未解锁
  #[error("数据库未解锁")]
  Locked,
  /// 操作失败
  #[error("操作失败")]
  Internal,
}

impl From<DbError> for DemoError {
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => DemoError::Locked,
      _ => DemoError::Internal,
    }
  }
}

impl Serialize for DemoError {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
