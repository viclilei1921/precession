use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use tauri::{
  plugin::{Builder, PluginHandle, TauriPlugin},
  Manager, Runtime,
};

const PLUGIN_IDENTIFIER: &str = "com.viclilei.precession.device";

#[derive(Debug, thiserror::Error)]
pub enum SlotError {
  #[error("unavailable")]
  Unavailable,
  #[error("cancelled")]
  Cancelled,
  #[error("invalid")]
  Invalid,
  #[error("internal")]
  Internal,
}

pub struct DeviceApi<R: Runtime> {
  handle: PluginHandle<R>,
}

impl<R: Runtime> Clone for DeviceApi<R> {
  /// 复制插件句柄
  ///
  /// # 返回值
  /// 持有同一句柄克隆的 `DeviceApi`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let again = api.clone();
  /// ```
  fn clone(&self) -> Self {
    Self { handle: self.handle.clone() }
  }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnrollRequest {
  dek_b64: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenRequest {
  ct_b64: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SlotResponse {
  ok: bool,
  code: Option<String>,
  ct_b64: Option<String>,
  dek_b64: Option<String>,
}

impl<R: Runtime> DeviceApi<R> {
  /// 把 DEK 的 Base64 交给移动端 `enroll`，再解回密文
  ///
  /// # 参数
  /// - `dek`: 32 字节 DEK
  ///
  /// # 返回值
  /// 插件返回的密文字节
  ///
  /// # 错误
  /// - 插件调用失败、没有 `ct_b64` 或 Base64 无法解码时返回 `Internal`
  /// - `ok` 为假时按 `code` 调用 `map_code`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let ct = api.enroll(dek)?;
  /// ```
  pub fn enroll(&self, dek: &[u8; 32]) -> Result<Vec<u8>, SlotError> {
    let response: SlotResponse = self
      .handle
      .run_mobile_plugin("enroll", EnrollRequest { dek_b64: BASE64.encode(dek) })
      .map_err(|_| SlotError::Internal)?;
    if !response.ok {
      return Err(map_code(response.code.as_deref()));
    }
    let ct_b64 = response.ct_b64.ok_or(SlotError::Internal)?;
    BASE64.decode(ct_b64).map_err(|_| SlotError::Internal)
  }

  /// 把密文的 Base64 交给移动端 `open`，再解回 DEK
  ///
  /// 解开的字节必须正好 32 字节。
  ///
  /// # 参数
  /// - `ct`: 平台密文
  ///
  /// # 返回值
  /// 32 字节 DEK
  ///
  /// # 错误
  /// - 插件调用失败、没有 `dek_b64` 或 Base64 无法解码时返回 `Internal`
  /// - `ok` 为假时按 `code` 调用 `map_code`
  /// - 解码后长度不是 32 时返回 `Invalid`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let dek = api.open(ct)?;
  /// ```
  pub fn open(&self, ct: &[u8]) -> Result<Vec<u8>, SlotError> {
    let response: SlotResponse = self
      .handle
      .run_mobile_plugin("open", OpenRequest { ct_b64: BASE64.encode(ct) })
      .map_err(|_| SlotError::Internal)?;
    if !response.ok {
      return Err(map_code(response.code.as_deref()));
    }
    let dek_b64 = response.dek_b64.ok_or(SlotError::Internal)?;
    let dek = BASE64.decode(dek_b64).map_err(|_| SlotError::Internal)?;
    if dek.len() != 32 {
      return Err(SlotError::Invalid);
    }
    Ok(dek)
  }

  /// 调用移动端 `confirm` 做系统验证
  ///
  /// # 返回值
  /// `ok` 为真时不返回数据
  ///
  /// # 错误
  /// 插件调用失败时返回 `Internal`；`ok` 为假时按 `code` 调用 `map_code`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// api.confirm()?;
  /// ```
  pub fn confirm(&self) -> Result<(), SlotError> {
    let response: SlotResponse = self.handle.run_mobile_plugin("confirm", ()).map_err(|_| SlotError::Internal)?;
    if response.ok {
      Ok(())
    } else {
      Err(map_code(response.code.as_deref()))
    }
  }

  /// 调用移动端 `forget` 删除设备密钥
  ///
  /// # 返回值
  /// `ok` 为真时不返回数据
  ///
  /// # 错误
  /// 插件调用失败时返回 `Internal`；`ok` 为假时按 `code` 调用 `map_code`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// api.forget()?;
  /// ```
  pub fn forget(&self) -> Result<(), SlotError> {
    let response: SlotResponse = self.handle.run_mobile_plugin("forget", ()).map_err(|_| SlotError::Internal)?;
    if response.ok {
      Ok(())
    } else {
      Err(map_code(response.code.as_deref()))
    }
  }
}

/// 把插件返回的代码字符串映射成 `SlotError`
///
/// # 参数
/// - `code`: 插件给出的代码；可以没有
///
/// # 返回值
/// `cancelled`、`unavailable`、`invalid` 对应同名变体，其他值返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(matches!(map_code(Some("cancelled")), SlotError::Cancelled));
/// assert!(matches!(map_code(None), SlotError::Internal));
/// ```
fn map_code(code: Option<&str>) -> SlotError {
  match code {
    Some("cancelled") => SlotError::Cancelled,
    Some("unavailable") => SlotError::Unavailable,
    Some("invalid") => SlotError::Invalid,
    _ => SlotError::Internal,
  }
}

/// 注册 Android 设备槽插件
///
/// 初始化时用 `com.viclilei.precession.device` 注册 `DeviceSlotPlugin`，并把 `DeviceApi` 放进应用状态。
///
/// # 返回值
/// 名为 `device-slot` 的 `TauriPlugin<R>`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let plugin = init::<tauri::Wry>();
/// ```
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("device-slot")
    .setup(|app, api| {
      let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "DeviceSlotPlugin")?;
      app.manage(DeviceApi { handle });
      Ok(())
    })
    .build()
}
