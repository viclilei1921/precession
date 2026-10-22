use serde::Serialize;

/// 版本读取失败
///
/// 序列化时只写出错误文本。读不到的单项仍写成占位文本，不走这个错误。
///
/// # 变体
/// - `Internal`: 读取失败
#[derive(Debug, thiserror::Error)]
pub enum AboutError {
  /// 读取失败
  #[expect(dead_code, reason = "单项失败写成占位文本，命令不返回这个错误")]
  #[error("读取失败")]
  Internal,
}

impl Serialize for AboutError {
  /// 把版本错误序列化成错误文本
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
  /// let text = serde_json::to_string(&AboutError::Internal);
  /// ```
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
