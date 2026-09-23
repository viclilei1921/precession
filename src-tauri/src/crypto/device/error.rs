/// 设备槽失败原因。`db` 把它翻译成给界面看的 `DbError`。
#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
  /// 本机没有可用的系统密钥库或用户验证
  #[error("这台设备不支持设备解锁")]
  Unavailable,
  /// 用户取消了系统验证
  #[error("已取消设备验证")]
  Cancelled,
  /// 密文损坏、平台不符，或系统里的设备密钥已经没了
  #[error("设备解锁已失效")]
  Invalid,
  /// 系统调用失败
  #[error("操作失败")]
  Internal,
}
