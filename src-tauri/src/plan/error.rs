use serde::Serialize;

use crate::db::error::DbError;

#[derive(Debug, thiserror::Error)]
pub enum PlanError {
  /// 标题不能为空
  #[error("标题不能为空")]
  TitleEmpty,
  /// 步骤标题不能为空
  #[error("步骤标题不能为空")]
  StepTitleEmpty,
  /// 状态不正确
  #[error("状态不正确")]
  StatusInvalid,
  /// 计划不存在
  #[error("计划不存在")]
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

impl From<DbError> for PlanError {
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => PlanError::Locked,
      _ => PlanError::Internal,
    }
  }
}

impl Serialize for PlanError {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
