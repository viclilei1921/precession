use serde::Serialize;

use crate::db::error::DbError;

#[derive(Debug, thiserror::Error)]
pub enum MemberError {
  /// 名称不能为空
  #[error("名称不能为空")]
  NameEmpty,
  /// 成员不存在
  #[error("成员不存在")]
  NotFound,
  /// 引用的成员不存在
  #[error("引用不存在")]
  ReferencedMissing,
  /// 数据库未解锁
  #[error("数据库未解锁")]
  Locked,
  /// 操作失败
  #[error("操作失败")]
  Internal,
}

impl From<DbError> for MemberError {
/// 把数据库错误转成成员错误
///
/// # 参数
/// - `err`: 数据库错误
///
/// # 返回值
/// 锁定时返回 `Locked`，其余返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let err = MemberError::from(DbError::Locked);
/// ```
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => MemberError::Locked,
      _ => MemberError::Internal,
    }
  }
}

impl Serialize for MemberError {
/// 把成员错误序列化成错误文本
///
/// # 参数
/// - `serializer`: 序列化器
///
/// # 返回值
/// 成功时返回序列化器的完成值
///
/// # 错误
/// 序列化器拒绝写入时返回其错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let text = serde_json::to_string(&MemberError::NotFound);
/// ```
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
