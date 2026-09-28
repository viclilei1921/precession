use serde::Serialize;

use crate::db::error::DbError;

#[derive(Debug, thiserror::Error)]
pub enum PlaceError {
  /// 名称不能为空
  #[error("名称不能为空")]
  NameEmpty,
  /// 地点不存在
  #[error("地点不存在")]
  NotFound,
  /// 引用的地点不存在
  #[error("引用不存在")]
  ReferencedMissing,
  /// 数据库未解锁
  #[error("数据库未解锁")]
  Locked,
  /// 操作失败
  #[error("操作失败")]
  Internal,
}

impl From<DbError> for PlaceError {
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => PlaceError::Locked,
      _ => PlaceError::Internal,
    }
  }
}

impl Serialize for PlaceError {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
