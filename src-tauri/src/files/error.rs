use serde::Serialize;

/// 收集文件失败
///
/// 序列化时只写出错误文本。
///
/// # 变体
/// - `Missing`: 路径不存在
/// - `Unreadable`: 路径存在但读不了
#[derive(Debug, thiserror::Error)]
pub enum FilesError {
  /// 路径不存在
  #[error("文件不存在")]
  Missing,
  /// 路径存在但读不了
  #[error("无法读取")]
  Unreadable,
}

impl Serialize for FilesError {
  /// 把收集错误序列化成错误文本
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
  /// ```ignore
  /// let text = serde_json::to_string(&FilesError::Missing);
  /// ```
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
