use serde::Serialize;

use crate::db::error::DbError;

#[derive(Debug, thiserror::Error)]
pub enum GrowthError {
  /// 标题不能为空
  #[error("标题不能为空")]
  TitleEmpty,
  /// 类型不正确
  #[error("类型不正确")]
  KindInvalid,
  /// 成长记录需要指定成员
  #[error("成长记录需要指定成员")]
  MemberRequired,
  /// 成长记录不存在
  #[error("成长记录不存在")]
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

impl From<DbError> for GrowthError {
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => GrowthError::Locked,
      _ => GrowthError::Internal,
    }
  }
}

impl Serialize for GrowthError {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
