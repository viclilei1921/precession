use serde::Serialize;

use crate::db::error::DbError;

/// 成长记录操作失败
///
/// 序列化时只写出错误文本。数据库锁定变成 `Locked`，其余数据库错误变成 `Internal`。
///
/// # 变体
/// - `TitleEmpty`: 标题不能为空
/// - `KindInvalid`: 类型不正确
/// - `MemberRequired`: 成长记录需要指定成员
/// - `NotFound`: 成长记录不存在
/// - `ReferencedMissing`: 引用不存在
/// - `Locked`: 数据库未解锁
/// - `Internal`: 操作失败
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
  /// 把数据库错误转成成长记录错误
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
  /// let err = GrowthError::from(db_err);
  /// ```
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => GrowthError::Locked,
      _ => GrowthError::Internal,
    }
  }
}

impl Serialize for GrowthError {
  /// 把成长记录错误序列化为错误文本
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
