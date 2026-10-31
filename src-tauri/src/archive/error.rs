use serde::Serialize;

/// 档案打包或恢复失败
///
/// 序列化时只写出错误文本。
///
/// # 变体
/// - `PathEmpty`: 没有选择位置
/// - `PathInvalid`: 保存位置不能放在数据目录里
/// - `Locked`: 数据库未解锁
/// - `Busy`: 还有任务在处理
/// - `Missing`: 数据库文件不存在
/// - `Invalid`: 压缩包不是本应用的档案
/// - `Io`: 读写文件失败
/// - `Internal`: 打包或恢复失败
/// - `Share`: 系统分享打不开
#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
  /// 没有选择位置
  #[error("没有选择位置")]
  PathEmpty,
  /// 保存位置不能放在数据目录里
  #[error("请选数据目录以外的位置")]
  PathInvalid,
  /// 数据库未解锁
  #[error("数据库未解锁")]
  Locked,
  /// 还有任务在处理
  #[error("还有任务在处理，完成后再试")]
  Busy,
  /// 数据库文件不存在
  #[error("数据库不存在")]
  Missing,
  /// 压缩包不是本应用的档案
  #[error("这不是可以导入的档案")]
  Invalid,
  /// 读写文件失败
  #[error("无法读写档案")]
  Io,
  /// 打包或恢复失败
  #[error("操作失败")]
  Internal,
  /// 系统分享打不开
  #[error("无法打开系统分享")]
  Share,
}

impl From<crate::db::error::DbError> for ArchiveError {
  /// 把数据库错误收成档案错误
  ///
  /// 未解锁保持为 `Locked`，读写失败保持为 `Io`，其余收成 `Internal`。
  ///
  /// # 参数
  /// - `err`: 数据库错误
  ///
  /// # 返回值
  /// 对应的 `ArchiveError`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let err = ArchiveError::from(crate::db::error::DbError::Locked);
  /// assert!(matches!(err, ArchiveError::Locked));
  /// ```
  fn from(err: crate::db::error::DbError) -> Self {
    match err {
      crate::db::error::DbError::Locked => ArchiveError::Locked,
      crate::db::error::DbError::Io => ArchiveError::Io,
      _ => ArchiveError::Internal,
    }
  }
}

impl Serialize for ArchiveError {
  /// 把错误显示文本序列化成字符串
  ///
  /// # 参数
  /// - `serializer`: serde 序列化器
  ///
  /// # 返回值
  /// 序列化成功时的 `S::Ok`
  ///
  /// # 错误
  /// `serialize_str` 失败时返回 `S::Error`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let json = serde_json::to_string(&ArchiveError::Locked).unwrap();
  /// assert_eq!(json, "\"数据库未解锁\"");
  /// ```
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
