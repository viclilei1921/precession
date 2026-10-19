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

/// 把平台名和密文收成 `device.wrap` 的 JSON
///
/// # 参数
/// - `kind`: 平台种类
/// - `ct`: 平台密文
///
/// # 返回值
/// 带版本、种类和 Base64 密文的 JSON 字节
///
/// # 错误
/// 序列化失败时返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let bytes = encode("memory", b"abc").unwrap();
/// assert!(bytes.starts_with(b"{"));
/// ```
pub(crate) fn encode(kind: &str, ct: &[u8]) -> Result<Vec<u8>, DeviceError> {
  let value = DeviceWrapJson { v: WRAP_VERSION, kind: kind.to_string(), ct: BASE64.encode(ct) };
  serde_json::to_vec_pretty(&value).map_err(|_| DeviceError::Internal)
}

/// 解析 `device.wrap` 并取出平台密文
///
/// 版本必须是 1，种类和密文都不能为空。Base64 会先去掉首尾空白。
///
/// # 参数
/// - `blob`: `device.wrap` 的全部字节
///
/// # 返回值
/// 平台种类和解码后的密文
///
/// # 错误
/// JSON 无法解析、版本或种类不合法、Base64 失败，或密文为空时返回 `Invalid`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let bytes = encode("memory", b"abc").unwrap();
/// let parsed = decode(&bytes).unwrap();
/// assert_eq!(parsed.kind, "memory");
/// assert_eq!(parsed.ct, b"abc");
/// ```
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
