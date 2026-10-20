/// 加密层失败原因。
///
/// db 会把它翻译成用户可读的 DbError。解密失败记成 `WrongPassword`；头字段、Base64 或长度对不上记成 `Corrupt`；随机源或加密调用失败记成 `Internal`。
///
/// # 变体
/// - `WrongPassword`: 密码不正确
/// - `Corrupt`: 密钥材料损坏
/// - `Internal`: 加密操作失败
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
