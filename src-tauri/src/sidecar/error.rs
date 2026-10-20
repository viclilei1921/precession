/// 侧车子进程的错误
///
/// # 变体
/// - `Canceled`: 调用方取消了这次侧车
/// - `Invalid`: 无效的参数，载荷是说明
/// - `Io`: 输入输出错误，载荷是说明
/// - `Spawn`: 侧车子进程启动失败，载荷是侧车名和原因
/// - `Exit`: 侧车子进程退出，载荷是侧车名和说明
#[derive(Debug, thiserror::Error)]
pub enum SidecarError {
  /// 调用方取消了这次侧车
  #[error("已取消")]
  Canceled,
  /// 无效的参数，载荷是说明
  #[error("{0}")]
  Invalid(String),
  /// 输入输出错误，载荷是说明
  #[error("{0}")]
  Io(String),
  /// 侧车子进程启动失败，载荷是侧车名和原因
  #[error("{name} 启动失败: {detail}")]
  Spawn { name: String, detail: String },
  /// 侧车子进程退出，载荷是侧车名和说明
  #[error("{name} {detail}")]
  Exit { name: String, detail: String },
}

impl SidecarError {
  /// 返回已取消错误
  ///
  /// # 返回值
  /// `SidecarError::Canceled`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let err = SidecarError::canceled();
  /// assert!(matches!(err, SidecarError::Canceled));
  /// ```
  pub fn canceled() -> Self {
    Self::Canceled
  }

  /// 用这段说明构造参数错误
  ///
  /// # 参数
  /// - `message`: 错误说明
  ///
  /// # 返回值
  /// `SidecarError::Invalid`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let err = SidecarError::invalid("没有音轨");
  /// assert!(matches!(err, SidecarError::Invalid(_)));
  /// ```
  pub fn invalid(message: impl Into<String>) -> Self {
    Self::Invalid(message.into())
  }

  /// 用这段说明构造输入输出错误
  ///
  /// # 参数
  /// - `message`: 错误说明
  ///
  /// # 返回值
  /// `SidecarError::Io`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let err = SidecarError::io("创建目录失败");
  /// assert!(matches!(err, SidecarError::Io(_)));
  /// ```
  pub fn io(message: impl Into<String>) -> Self {
    Self::Io(message.into())
  }

  /// 记下侧车启动失败
  ///
  /// # 参数
  /// - `name`: 侧车名
  /// - `err`: 启动失败原因
  ///
  /// # 返回值
  /// `SidecarError::Spawn`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let err = SidecarError::sidecar_spawn("ffmpeg", "missing");
  /// assert!(matches!(err, SidecarError::Spawn { .. }));
  /// ```
  pub fn sidecar_spawn(name: impl Into<String>, err: impl std::fmt::Display) -> Self {
    Self::Spawn { name: name.into(), detail: err.to_string() }
  }

  /// 记下侧车退出时的说明
  ///
  /// # 参数
  /// - `name`: 侧车名
  /// - `detail`: 退出说明
  ///
  /// # 返回值
  /// `SidecarError::Exit`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let err = SidecarError::sidecar_exit("ffmpeg", "退出代码 1");
  /// assert!(matches!(err, SidecarError::Exit { .. }));
  /// ```
  pub fn sidecar_exit(name: impl Into<String>, detail: impl Into<String>) -> Self {
    Self::Exit { name: name.into(), detail: detail.into() }
  }
}

/// 从 `std::io::Error` 创建一个侧车子进程的错误
impl From<std::io::Error> for SidecarError {
  /// 把输入输出错误的显示文本放进 `Io`
  ///
  /// # 参数
  /// - `err`: 输入输出错误
  ///
  /// # 返回值
  /// `SidecarError::Io`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let err = SidecarError::from(std::io::Error::other("fail"));
  /// assert!(matches!(err, SidecarError::Io(_)));
  /// ```
  fn from(err: std::io::Error) -> Self {
    Self::Io(err.to_string())
  }
}

/// 侧车子进程的结果
///
/// 指向 `Result<T, SidecarError>`，让侧车函数的签名不用重复错误类型。
pub type SidecarResult<T> = Result<T, SidecarError>;
