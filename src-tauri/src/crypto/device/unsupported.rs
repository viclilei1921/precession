use zeroize::Zeroizing;

use super::error::DeviceError;

/// 没有系统密钥库的设备槽
///
/// 登记和打开一律返回 `Unavailable`。忘记设备密钥直接成功。
///
/// # 示例
/// ```ignore
/// assert!(matches!(UnsupportedDevice.enroll(&[7u8; 32]), Err(DeviceError::Unavailable)));
/// UnsupportedDevice.forget()?;
/// ```
pub(crate) struct UnsupportedDevice;

impl super::DeviceSlot for UnsupportedDevice {
  /// 拒绝登记，因为这个平台没有设备槽
  ///
  /// # 参数
  /// - `_dek`: 32 字节 DEK，此处不使用
  ///
  /// # 返回值
  /// 不返回成功值
  ///
  /// # 错误
  /// 始终返回 `Unavailable`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let err = slot.enroll(dek).unwrap_err();
  /// ```
  fn enroll(&self, _dek: &[u8; 32]) -> Result<Vec<u8>, DeviceError> {
    Err(DeviceError::Unavailable)
  }

  /// 拒绝打开，因为这个平台没有设备槽
  ///
  /// # 参数
  /// - `_blob`: `device.wrap` 字节，此处不使用
  ///
  /// # 返回值
  /// 不返回成功值
  ///
  /// # 错误
  /// 始终返回 `Unavailable`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let err = slot.open(blob).unwrap_err();
  /// ```
  fn open(&self, _blob: &[u8]) -> Result<Zeroizing<[u8; 32]>, DeviceError> {
    Err(DeviceError::Unavailable)
  }

  /// 在没有设备槽的平台上直接成功返回
  ///
  /// # 返回值
  /// 成功时不返回数据
  ///
  /// # 错误
  /// 不会返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// slot.forget()?;
  /// ```
  fn forget(&self) -> Result<(), DeviceError> {
    Ok(())
  }
}
