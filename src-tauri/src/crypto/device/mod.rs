mod error;
#[cfg(test)]
mod memory;
mod wrap;

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

use std::sync::Arc;

use zeroize::Zeroizing;

#[cfg(target_os = "android")]
pub(crate) use android::AndroidDevice;
pub(crate) use error::DeviceError;
#[cfg(test)]
pub(crate) use memory::MemorySlot;

/// 用系统密钥库包住 DEK。
///
/// 返回值是 `device.wrap` 的全部字节。实现者必须是 `Send` + `Sync`，才能放进跨线程共享的槽。`confirm` 的默认实现直接成功。
pub(crate) trait DeviceSlot: Send + Sync {
  /// 把 DEK 交给系统密钥库并返回 `device.wrap` 的全部字节
  ///
  /// # 参数
  /// - `dek`: 32 字节 DEK
  ///
  /// # 返回值
  /// 成功时返回可写入 `device.wrap` 的字节
  ///
  /// # 错误
  /// 登记失败时返回 `Err(DeviceError)`，具体变体由实现决定
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let blob = slot.enroll(dek)?;
  /// ```
  fn enroll(&self, dek: &[u8; 32]) -> Result<Vec<u8>, DeviceError>;
  /// 用 `device.wrap` 的字节解开 DEK
  ///
  /// # 参数
  /// - `blob`: `device.wrap` 的全部字节
  ///
  /// # 返回值
  /// 成功时返回会在丢弃时清零的 DEK
  ///
  /// # 错误
  /// 打开失败时返回 `Err(DeviceError)`，具体变体由实现决定
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let dek = slot.open(blob)?;
  /// ```
  fn open(&self, blob: &[u8]) -> Result<Zeroizing<[u8; 32]>, DeviceError>;
  /// 删除系统密钥库里的设备密钥
  ///
  /// # 返回值
  /// 成功时不返回数据
  ///
  /// # 错误
  /// 删除失败时返回 `Err(DeviceError)`，具体变体由实现决定
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// slot.forget()?;
  /// ```
  fn forget(&self) -> Result<(), DeviceError>;

  /// 默认直接通过系统验证
  ///
  /// # 返回值
  /// 成功时不返回数据
  ///
  /// # 错误
  /// 默认实现不会返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// slot.confirm()?;
  /// ```
  fn confirm(&self) -> Result<(), DeviceError> {
    Ok(())
  }
}

/// 返回当前平台的设备槽
///
/// Windows 使用 `WindowsDevice`，macOS 使用 `MacosDevice`，其余目标使用 `UnsupportedDevice`。
///
/// # 返回值
/// 包在 `Arc` 里的设备槽
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let slot = platform_slot();
/// ```
pub(crate) fn platform_slot() -> Arc<dyn DeviceSlot> {
  #[cfg(target_os = "windows")]
  {
    Arc::new(windows::WindowsDevice)
  }
  #[cfg(target_os = "macos")]
  {
    Arc::new(macos::MacosDevice)
  }
  #[cfg(not(any(target_os = "windows", target_os = "macos")))]
  {
    Arc::new(unsupported::UnsupportedDevice)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn memory_slot_roundtrip_and_isolation() {
    let slot = MemorySlot::new();
    let dek = [7u8; 32];
    let blob = slot.enroll(&dek).unwrap();
    assert_eq!(slot.open(&blob).unwrap().as_ref(), &dek);

    let again = slot.enroll(&dek).unwrap();
    assert!(slot.open(&blob).is_err());
    assert_eq!(slot.open(&again).unwrap().as_ref(), &dek);

    let other = MemorySlot::new();
    assert!(matches!(other.open(&again), Err(DeviceError::Invalid)));

    slot.forget().unwrap();
    assert!(matches!(slot.open(&again), Err(DeviceError::Invalid)));
    assert!(matches!(slot.open(b"not-json"), Err(DeviceError::Invalid)));
  }
}
