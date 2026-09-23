//! Android 设备槽走应用内插件。DEK 只在进程内交给 Keystore，不进 WebView。

use zeroize::Zeroizing;

use super::error::DeviceError;
use super::wrap;

const KIND_ANDROID: &str = "android-keystore";
use precession_device_slot::{DeviceApi, SlotError};

pub(crate) struct AndroidDevice {
  api: DeviceApi<tauri::Wry>,
}

impl AndroidDevice {
  pub(crate) fn new(api: DeviceApi<tauri::Wry>) -> Self {
    Self { api }
  }
}

impl super::DeviceSlot for AndroidDevice {
  fn enroll(&self, dek: &[u8; 32]) -> Result<Vec<u8>, DeviceError> {
    let ct = self.api.enroll(dek).map_err(map_slot)?;
    wrap::encode(KIND_ANDROID, &ct)
  }

  fn open(&self, blob: &[u8]) -> Result<Zeroizing<[u8; 32]>, DeviceError> {
    let parsed = wrap::decode(blob)?;
    if parsed.kind != KIND_ANDROID {
      return Err(DeviceError::Invalid);
    }
    let dek = self.api.open(&parsed.ct).map_err(map_slot)?;
    let dek: [u8; 32] = dek.as_slice().try_into().map_err(|_| DeviceError::Invalid)?;
    Ok(Zeroizing::new(dek))
  }

  fn confirm(&self) -> Result<(), DeviceError> {
    self.api.confirm().map_err(map_slot)
  }

  fn forget(&self) -> Result<(), DeviceError> {
    self.api.forget().map_err(map_slot)
  }
}

fn map_slot(err: SlotError) -> DeviceError {
  match err {
    SlotError::Unavailable => DeviceError::Unavailable,
    SlotError::Cancelled => DeviceError::Cancelled,
    SlotError::Invalid => DeviceError::Invalid,
    SlotError::Internal => DeviceError::Internal,
  }
}
