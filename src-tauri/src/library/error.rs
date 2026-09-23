use serde::Serialize;

use crate::db::error::DbError;

#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
  /// 标题不能为空
  #[error("标题不能为空")]
  TitleEmpty,
  /// 状态不正确
  #[error("状态不正确")]
  StatusInvalid,
  /// 进度不正确
  #[error("进度不正确")]
  ProgressInvalid,
  /// 类型不正确
  #[error("类型不正确")]
  TypeInvalid,
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

impl From<DbError> for LibraryError {
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => LibraryError::Locked,
      _ => LibraryError::Internal,
    }
  }
}

impl Serialize for LibraryError {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
