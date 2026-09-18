/// 加密层失败原因。db 会把它翻译成用户可读的 DbError。
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
  /// 密码不正确
  #[error("密码不正确")]
  WrongPassword,
  /// 密钥材料损坏
  #[error("密钥材料损坏")]
  Corrupt,
  /// 加密操作失败
  #[error("加密操作失败")]
  Internal,
}
