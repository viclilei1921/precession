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

/// 用系统密钥库包住 DEK。返回值是 `device.wrap` 的全部字节。
pub(crate) trait DeviceSlot: Send + Sync {
  fn enroll(&self, dek: &[u8; 32]) -> Result<Vec<u8>, DeviceError>;
  fn open(&self, blob: &[u8]) -> Result<Zeroizing<[u8; 32]>, DeviceError>;
  fn forget(&self) -> Result<(), DeviceError>;

  /// 用户锁定之后再解锁时，先通过系统验证。默认直接通过，测试用假槽走这里。
  fn confirm(&self) -> Result<(), DeviceError> {
    Ok(())
  }
}

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
