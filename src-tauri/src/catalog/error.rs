use serde::Serialize;

use crate::db::error::DbError;

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
  /// 名称不能为空
  #[error("名称不能为空")]
  NameEmpty,
  /// 名称已被使用
  #[error("名称已被使用")]
  NameTaken,
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

impl From<DbError> for CatalogError {
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => CatalogError::Locked,
      _ => CatalogError::Internal,
    }
  }
}

impl Serialize for CatalogError {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
