use serde::Serialize;

use crate::db::error::DbError;

#[derive(Debug, thiserror::Error)]
pub enum MediaError {
  /// 类型不正确
  #[error("类型不正确")]
  KindInvalid,
  /// 媒体不存在
  #[error("媒体不存在")]
  NotFound,
  /// 所属记录不存在
  #[error("引用不存在")]
  ReferencedMissing,
  /// 数据库未解锁
  #[error("数据库未解锁")]
  Locked,
  /// 操作失败
  #[error("操作失败")]
  Internal,
}

impl From<DbError> for MediaError {
  /// 把数据库错误转成媒体错误
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
  /// let err = MediaError::from(db_err);
  /// ```
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => MediaError::Locked,
      _ => MediaError::Internal,
    }
  }
}

impl Serialize for MediaError {
  /// 把媒体错误序列化为错误文本
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
