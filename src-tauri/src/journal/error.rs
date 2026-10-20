use serde::Serialize;

use crate::db::error::DbError;

/// 手记操作失败
///
/// 序列化时只写出错误文本。数据库锁定变成 `Locked`，其余数据库错误变成 `Internal`。
///
/// # 变体
/// - `TitleEmpty`: 标题不能为空
/// - `KindInvalid`: 类型不正确
/// - `NotFound`: 手记不存在
/// - `ReferencedMissing`: 引用不存在
/// - `Locked`: 数据库未解锁
/// - `PasswordEmpty`: 密码不能为空
/// - `BodyUnreadable`: 正文无法读取
/// - `Internal`: 操作失败
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
  /// 密码不能为空
  #[error("密码不能为空")]
  PasswordEmpty,
  /// 正文无法读取
  #[error("正文无法读取")]
  BodyUnreadable,
  /// 操作失败
  #[error("操作失败")]
  Internal,
}

impl From<DbError> for JournalError {
/// 把数据库错误转成手记错误
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
/// let err = JournalError::from(DbError::Locked);
/// ```
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => JournalError::Locked,
      _ => JournalError::Internal,
    }
  }
}

impl Serialize for JournalError {
/// 把手记错误序列化成错误文本
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
/// let text = serde_json::to_string(&JournalError::NotFound);
/// ```
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
