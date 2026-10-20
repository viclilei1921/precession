use serde::Serialize;

use crate::crypto::device::DeviceError;
use crate::crypto::error::CryptoError;

/// 数据库操作失败
///
/// 序列化时只写出错误文本。设备槽的 `Unavailable`、`Cancelled`、`Invalid`、`Internal` 分别变成 `DeviceUnavailable`、`DeviceCancelled`、`DeviceInvalid`、`Internal`。加密层的 `WrongPassword`、`Corrupt`、`Internal` 变成同名变体。
///
/// # 变体
/// - `WrongPassword`: 密码不正确
/// - `PasswordMismatch`: 两次输入的密码不一致
/// - `PasswordEmpty`: 密码不能为空
/// - `PasswordTooShort`: 密码太短
/// - `AlreadyExists`: 数据库已存在
/// - `NotFound`: 数据库不存在
/// - `Corrupt`: 数据库损坏
/// - `Locked`: 数据库未解锁
/// - `Io`: 无法访问数据目录
/// - `Internal`: 操作失败
/// - `AlreadyUnlocked`: 数据库已解锁
/// - `DeviceUnavailable`: 本机没有 TPM、钥匙串或 Android Keystore
/// - `DeviceCancelled`: 用户取消了系统验证
/// - `DeviceInvalid`: 设备槽密文或系统密钥已经对不上
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
  /// 密码太短
  #[error("密码至少 8 位")]
  PasswordTooShort,
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
  /// 把设备槽错误映射成数据库错误
  ///
  /// `Unavailable`、`Cancelled`、`Invalid`、`Internal` 分别对应同名含义的 `DbError` 变体。
  ///
  /// # 参数
  /// - `err`: 设备槽错误
  ///
  /// # 返回值
  /// 对应的 `DbError`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let err = DbError::from(DeviceError::Cancelled);
  /// assert!(matches!(err, DbError::DeviceCancelled));
  /// ```
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
  /// 把加密层错误映射成数据库错误
  ///
  /// `WrongPassword`、`Corrupt`、`Internal` 分别对应同名的 `DbError` 变体。
  ///
  /// # 参数
  /// - `err`: 加密层错误
  ///
  /// # 返回值
  /// 对应的 `DbError`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let err = DbError::from(CryptoError::WrongPassword);
  /// assert!(matches!(err, DbError::WrongPassword));
  /// ```
  fn from(err: CryptoError) -> Self {
    match err {
      CryptoError::WrongPassword => DbError::WrongPassword,
      CryptoError::Corrupt => DbError::Corrupt,
      CryptoError::Internal => DbError::Internal,
    }
  }
}

impl Serialize for DbError {
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
  /// let json = serde_json::to_string(&DbError::Locked).unwrap();
  /// assert_eq!(json, "\"数据库未解锁\"");
  /// ```
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
