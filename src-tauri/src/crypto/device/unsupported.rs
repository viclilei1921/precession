use zeroize::Zeroizing;

use super::error::DeviceError;

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
