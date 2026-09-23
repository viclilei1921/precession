use serde::Serialize;

use crate::crypto::device::DeviceError;
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
  /// 本机没有 TPM、钥匙串或 Android Keystore
  #[error("这台设备不支持设备解锁")]
  DeviceUnavailable,
  /// 用户取消了系统验证
  #[error("已取消设备验证")]
  DeviceCancelled,
  /// 设备槽密文或系统密钥已经对不上
  #[error("设备解锁已失效")]
  DeviceInvalid,
}

impl From<DeviceError> for DbError {
  fn from(err: DeviceError) -> Self {
    match err {
      DeviceError::Unavailable => DbError::DeviceUnavailable,
      DeviceError::Cancelled => DbError::DeviceCancelled,
      DeviceError::Invalid => DbError::DeviceInvalid,
      DeviceError::Internal => DbError::Internal,
    }
  }
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
