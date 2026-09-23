use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};

use super::error::DeviceError;

pub(crate) const WRAP_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct DeviceWrapJson {
  v: u32,
  kind: String,
  ct: String,
}

pub(crate) struct ParsedWrap {
  pub kind: String,
  pub ct: Vec<u8>,
}

/// 把平台密文收成 `device.wrap` 的 JSON。不含 DEK，不含密码。
pub(crate) fn encode(kind: &str, ct: &[u8]) -> Result<Vec<u8>, DeviceError> {
  let value = DeviceWrapJson { v: WRAP_VERSION, kind: kind.to_string(), ct: BASE64.encode(ct) };
  serde_json::to_vec_pretty(&value).map_err(|_| DeviceError::Internal)
}

pub(crate) fn decode(blob: &[u8]) -> Result<ParsedWrap, DeviceError> {
  let value: DeviceWrapJson = serde_json::from_slice(blob).map_err(|_| DeviceError::Invalid)?;
  if value.v != WRAP_VERSION || value.kind.is_empty() {
    return Err(DeviceError::Invalid);
  }
  let ct = BASE64.decode(value.ct.trim()).map_err(|_| DeviceError::Invalid)?;
  if ct.is_empty() {
    return Err(DeviceError::Invalid);
  }
  Ok(ParsedWrap { kind: value.kind, ct })
}
