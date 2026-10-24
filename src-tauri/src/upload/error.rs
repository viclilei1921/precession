use serde::Serialize;

use crate::db::error::DbError;

/// 上传文件操作失败
///
/// 序列化时只写出错误文本。数据库锁定变成 `Locked`，其余数据库错误变成 `Internal`。
///
/// # 变体
/// - `KindInvalid`: 类型不正确
/// - `NotFound`: 文件记录不存在
/// - `ReferencedMissing`: 所属记录不存在
/// - `Encrypted`: 文件已经加密
/// - `TooLarge`: 文件太大
/// - `Locked`: 数据库未解锁
/// - `Internal`: 操作失败
#[derive(Debug, thiserror::Error)]
pub enum UploadError {
  /// 类型不正确
  #[error("类型不正确")]
  KindInvalid,
  /// 文件记录不存在
  #[error("文件记录不存在")]
  NotFound,
  /// 所属记录不存在
  #[error("引用不存在")]
  ReferencedMissing,
  /// 文件已经加密
  #[error("文件已经加密")]
  Encrypted,
  /// 文件太大
  #[error("文件太大")]
  TooLarge,
  /// 数据库未解锁
  #[error("数据库未解锁")]
  Locked,
  /// 操作失败
  #[error("操作失败")]
  Internal,
}

impl From<DbError> for UploadError {
  /// 把数据库错误转成上传文件错误
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
  /// let err = UploadError::from(db_err);
  /// ```
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => UploadError::Locked,
      _ => UploadError::Internal,
    }
  }
}

impl Serialize for UploadError {
  /// 把上传文件错误序列化为错误文本
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
