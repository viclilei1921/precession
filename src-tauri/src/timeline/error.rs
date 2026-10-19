use serde::Serialize;

use crate::db::error::DbError;

#[derive(Debug, thiserror::Error)]
pub enum TimelineError {
  /// 类型不正确
  #[error("类型不正确")]
  KindInvalid,
  /// 数据库未解锁
  #[error("数据库未解锁")]
  Locked,
  /// 操作失败
  #[error("操作失败")]
  Internal,
}

impl From<DbError> for TimelineError {
  /// 把数据库错误转成时间线错误
  ///
  /// # 参数
  /// - `err`: 数据库错误
  ///
  /// # 返回值
  /// 数据库未解锁时返回未解锁，其余返回内部错误
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let err = TimelineError::from(db_err);
  /// ```
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => TimelineError::Locked,
      _ => TimelineError::Internal,
    }
  }
}

impl Serialize for TimelineError {
  /// 把时间线错误序列化为错误文本
  ///
  /// # 参数
  /// - `serializer`: 序列化器
  ///
  /// # 返回值
  /// 返回序列化器写入后的结果
  ///
  /// # 错误
  /// 序列化失败时返回序列化器错误
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let value = err.serialize(serializer);
  /// ```
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
