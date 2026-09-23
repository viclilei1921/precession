use std::sync::Mutex;

use zeroize::Zeroizing;

use super::error::DeviceError;
use super::wrap;

const KIND_MEMORY: &str = "memory";
use crate::crypto::constants::NONCE_LEN;
use crate::crypto::dek::{random_dek, unwrap_dek, wrap_dek};

/// 测试用设备槽。KEK 只在内存里，所以另一只槽解不开这份密文。
pub(crate) struct MemorySlot {
  kek: Mutex<Option<Zeroizing<[u8; 32]>>>,
}

impl MemorySlot {
  pub(crate) fn new() -> Self {
    Self { kek: Mutex::new(None) }
  }
}

impl super::DeviceSlot for MemorySlot {
  fn enroll(&self, dek: &[u8; 32]) -> Result<Vec<u8>, DeviceError> {
    let kek = random_dek().map_err(|_| DeviceError::Internal)?;
    let (nonce, ct) = wrap_dek(&kek, dek).map_err(|_| DeviceError::Internal)?;
    let mut packed = Vec::with_capacity(NONCE_LEN + ct.len());
    packed.extend_from_slice(&nonce);
    packed.extend_from_slice(&ct);
    *self.kek.lock().unwrap() = Some(kek);
    wrap::encode(KIND_MEMORY, &packed)
  }

  fn open(&self, blob: &[u8]) -> Result<Zeroizing<[u8; 32]>, DeviceError> {
    let parsed = wrap::decode(blob)?;
    if parsed.kind != KIND_MEMORY || parsed.ct.len() <= NONCE_LEN {
      return Err(DeviceError::Invalid);
    }
    let kek = self.kek.lock().unwrap().clone().ok_or(DeviceError::Invalid)?;
    let (nonce, ct) = parsed.ct.split_at(NONCE_LEN);
    unwrap_dek(&kek, nonce, ct).map_err(|_| DeviceError::Invalid)
  }

  fn forget(&self) -> Result<(), DeviceError> {
    *self.kek.lock().unwrap() = None;
    Ok(())
  }
}
