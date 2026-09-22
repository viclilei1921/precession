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

#[derive(Clone)]
pub struct DeviceApi<R: Runtime> {
  handle: PluginHandle<R>,
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

  pub fn confirm(&self) -> Result<(), SlotError> {
    let response: SlotResponse = self.handle.run_mobile_plugin("confirm", ()).map_err(|_| SlotError::Internal)?;
    if response.ok {
      Ok(())
    } else {
      Err(map_code(response.code.as_deref()))
    }
  }

  pub fn forget(&self) -> Result<(), SlotError> {
    let response: SlotResponse = self.handle.run_mobile_plugin("forget", ()).map_err(|_| SlotError::Internal)?;
    if response.ok {
      Ok(())
    } else {
      Err(map_code(response.code.as_deref()))
    }
  }
}

fn map_code(code: Option<&str>) -> SlotError {
  match code {
    Some("cancelled") => SlotError::Cancelled,
    Some("unavailable") => SlotError::Unavailable,
    Some("invalid") => SlotError::Invalid,
    _ => SlotError::Internal,
  }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("device-slot")
    .setup(|app, api| {
      let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "DeviceSlotPlugin")?;
      app.manage(DeviceApi { handle });
      Ok(())
    })
    .build()
}
