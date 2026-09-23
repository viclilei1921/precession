use serde::Serialize;

use crate::db::error::DbError;

#[derive(Debug, thiserror::Error)]
pub enum RecordError {
  /// 标题不能为空
  #[error("标题不能为空")]
  TitleEmpty,
  /// 类型不正确
  #[error("类型不正确")]
  TypeInvalid,
  /// 成长记录需要指定成员
  #[error("成长记录需要指定成员")]
  SubjectRequired,
  /// 这条记录不能指定成长主体
  #[error("这条记录不能指定成长主体")]
  SubjectNotAllowed,
  /// 引用不存在
  #[error("引用不存在")]
  ReferencedMissing,
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

impl From<DbError> for RecordError {
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => RecordError::Locked,
      _ => RecordError::Internal,
    }
  }
}

impl Serialize for RecordError {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
