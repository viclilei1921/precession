use serde::Serialize;

/// 看图失败
///
/// 序列化时只写出错误文本，前端按这句话翻译。
///
/// # 变体
/// - `Missing`: 路径不存在
/// - `Unreadable`: 路径存在但读不了
/// - `Unsupported`: 不是查看器接受的图片
/// - `Decode`: JPEG XL 解不成可以显示的图
/// - `WindowsOnly`: 当前系统不能注册默认打开方式
/// - `Settings`: 系统的默认应用设置打不开
#[derive(Debug, thiserror::Error)]
pub enum ViewerError {
  /// 路径不存在
  #[error("文件不存在")]
  Missing,
  /// 路径存在但读不了
  #[error("无法读取")]
  Unreadable,
  /// 不是查看器接受的图片
  #[error("不支持的文件格式")]
  Unsupported,
  /// JPEG XL 解不成可以显示的图
  #[error("无法解码图片")]
  Decode,
  /// 当前系统不能注册默认打开方式。Windows 上不会构造这个变体
  #[cfg_attr(windows, allow(dead_code))]
  #[error("仅 Windows 可以注册默认打开方式")]
  WindowsOnly,
  /// 系统的默认应用设置打不开
  #[error("无法打开默认应用设置")]
  Settings,
}

impl Serialize for ViewerError {
  /// 把看图错误序列化成错误文本
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
  /// let text = serde_json::to_string(&ViewerError::Unsupported);
  /// ```
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
