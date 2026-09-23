use zeroize::Zeroizing;

use super::error::DeviceError;

pub(crate) struct UnsupportedDevice;

impl super::DeviceSlot for UnsupportedDevice {
  fn enroll(&self, _dek: &[u8; 32]) -> Result<Vec<u8>, DeviceError> {
    Err(DeviceError::Unavailable)
  }

  fn open(&self, _blob: &[u8]) -> Result<Zeroizing<[u8; 32]>, DeviceError> {
    Err(DeviceError::Unavailable)
  }

  fn forget(&self) -> Result<(), DeviceError> {
    Ok(())
  }
}
