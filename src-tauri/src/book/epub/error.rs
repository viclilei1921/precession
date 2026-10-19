//! EPUB 解析错误类型

/// EPUB 解析错误
#[derive(Debug)]
pub struct EpubError(pub String);

impl std::fmt::Display for EpubError {
  /// 写入内部错误文本
  ///
  /// # 参数
  /// - `f`: 输出格式化器
  ///
  /// # 返回值
  /// 写入成功时返回 `Ok(())`
  ///
  /// # 错误
  /// 写入失败时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let err = EpubError("container.xml 解析失败".to_string());
  /// assert_eq!(err.to_string(), "container.xml 解析失败");
  /// ```
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl std::error::Error for EpubError {}
