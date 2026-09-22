use serde::Serialize;

use crate::crypto::error::CryptoError;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
  /// 密码不正确
  #[error("密码不正确")]
  WrongPassword,
  /// 两次输入的密码不一致
  #[error("两次输入的密码不一致")]
  PasswordMismatch,
  /// 密码不能为空
  #[error("密码不能为空")]
  PasswordEmpty,
  /// 数据库已存在
  #[error("数据库已存在")]
  AlreadyExists,
  /// 数据库不存在
  #[error("数据库不存在")]
  NotFound,
  /// 数据库损坏
  #[error("数据库损坏")]
  Corrupt,
  /// 数据库未解锁
  #[error("数据库未解锁")]
  Locked,
  /// 无法访问数据目录
  #[error("无法访问数据目录")]
  Io,
  /// 操作失败
  #[error("操作失败")]
  Internal,
  /// 数据库已解锁
  #[error("数据库已解锁")]
  AlreadyUnlocked,
}

impl From<CryptoError> for DbError {
  fn from(err: CryptoError) -> Self {
    match err {
      CryptoError::WrongPassword => DbError::WrongPassword,
      CryptoError::Corrupt => DbError::Corrupt,
      CryptoError::Internal => DbError::Internal,
    }
  }
}

impl Serialize for DbError {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
