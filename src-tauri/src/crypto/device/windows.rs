//! TPM 上不可导出的 RSA 密钥包装 DEK。日常打开直接解密。
//! 用户锁定之后再解锁时，先弹出 Windows Hello。
//!
//! Hello 由本进程发起。同一已登录用户里的其他程序仍可能直接调用这把 TPM 密钥。
//! 拷走 `device.wrap` 和数据库文件解不开，因为私钥不能导出。

use std::ptr;

use windows::Security::Credentials::UI::{
  UserConsentVerificationResult, UserConsentVerifier, UserConsentVerifierAvailability,
};
use windows::Win32::Security::Cryptography::{
  BCRYPT_OAEP_PADDING_INFO, BCRYPT_SHA256_ALGORITHM, CERT_KEY_SPEC, NCRYPT_ALLOW_DECRYPT_FLAG, NCRYPT_FLAGS,
  NCRYPT_KEY_HANDLE, NCRYPT_PAD_OAEP_FLAG, NCRYPT_PROV_HANDLE, NCRYPT_SILENT_FLAG, NCryptCreatePersistedKey,
  NCryptDecrypt, NCryptDeleteKey, NCryptEncrypt, NCryptFinalizeKey, NCryptFreeObject, NCryptOpenKey,
  NCryptOpenStorageProvider, NCryptSetProperty,
};
use windows::Win32::System::WinRT::IUserConsentVerifierInterop;
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
use windows::core::{HSTRING, PCWSTR};
use zeroize::Zeroizing;

use super::error::DeviceError;
use super::wrap;

const KIND_WINDOWS: &str = "windows-cng";

const KEY_NAME: &str = "precession.device.v1";
const PROVIDER_NAME: &str = "Microsoft Platform Crypto Provider";

pub(crate) struct WindowsDevice;

impl super::DeviceSlot for WindowsDevice {
  fn enroll(&self, dek: &[u8; 32]) -> Result<Vec<u8>, DeviceError> {
    let ct = seal_with_tpm(dek)?;
    wrap::encode(KIND_WINDOWS, &ct)
  }

  fn open(&self, blob: &[u8]) -> Result<Zeroizing<[u8; 32]>, DeviceError> {
    let parsed = wrap::decode(blob)?;
    if parsed.kind != KIND_WINDOWS {
      return Err(DeviceError::Invalid);
    }
    open_with_tpm(&parsed.ct)
  }

  fn confirm(&self) -> Result<(), DeviceError> {
    verify_hello()
  }

  fn forget(&self) -> Result<(), DeviceError> {
    let provider = open_provider()?;
    match open_key(&provider, true) {
      Ok(key) => delete_key(key),
      Err(DeviceError::Invalid) => Ok(()),
      Err(err) => Err(err),
    }
  }
}

fn verify_hello() -> Result<(), DeviceError> {
  let availability = UserConsentVerifier::CheckAvailabilityAsync()
    .map_err(|err| {
      log_windows("hello availability", &err);
      DeviceError::Unavailable
    })?
    .get()
    .map_err(|err| {
      log_windows("hello availability get", &err);
      DeviceError::Unavailable
    })?;
  if availability != UserConsentVerifierAvailability::Available {
    return Err(DeviceError::Unavailable);
  }

  let hwnd = unsafe { GetForegroundWindow() };
  if hwnd.0.is_null() {
    return Err(DeviceError::Unavailable);
  }

  let interop: IUserConsentVerifierInterop =
    windows::core::factory::<UserConsentVerifier, IUserConsentVerifierInterop>().map_err(|err| {
      log_windows("hello interop", &err);
      DeviceError::Unavailable
    })?;
  let operation: windows_future::IAsyncOperation<UserConsentVerificationResult> = unsafe {
    interop.RequestVerificationForWindowAsync(hwnd, &HSTRING::from("解锁人生档案")).map_err(|err| {
      log_windows("hello request", &err);
      DeviceError::Unavailable
    })?
  };
  let result = operation.get().map_err(|err| {
    log_windows("hello get", &err);
    DeviceError::Internal
  })?;
  match result {
    UserConsentVerificationResult::Verified => Ok(()),
    UserConsentVerificationResult::Canceled => Err(DeviceError::Cancelled),
    _ => Err(DeviceError::Unavailable),
  }
}

fn seal_with_tpm(dek: &[u8; 32]) -> Result<Vec<u8>, DeviceError> {
  let provider = open_provider()?;
  if let Ok(existing) = open_key(&provider, true) {
    delete_key(existing)?;
  }
  let key = create_key(&provider)?;
  match encrypt(&key, dek) {
    Ok(ct) => Ok(ct),
    Err(err) => {
      let _ = delete_key(key);
      Err(err)
    }
  }
}

fn open_with_tpm(ct: &[u8]) -> Result<Zeroizing<[u8; 32]>, DeviceError> {
  let provider = open_provider()?;
  let key = open_key(&provider, true)?;
  decrypt(&key, ct)
}

struct Provider(NCRYPT_PROV_HANDLE);

impl Drop for Provider {
  fn drop(&mut self) {
    if !self.0.is_invalid() {
      unsafe {
        let _ = NCryptFreeObject(self.0.into());
      }
    }
  }
}

struct Key(Option<NCRYPT_KEY_HANDLE>);

impl Drop for Key {
  fn drop(&mut self) {
    if let Some(handle) = self.0.take() {
      unsafe {
        let _ = NCryptFreeObject(handle.into());
      }
    }
  }
}

fn open_provider() -> Result<Provider, DeviceError> {
  let mut provider = NCRYPT_PROV_HANDLE::default();
  let name = HSTRING::from(PROVIDER_NAME);
  unsafe { NCryptOpenStorageProvider(&mut provider, PCWSTR(name.as_ptr()), 0) }.map_err(|err| {
    log_windows("open provider", &err);
    DeviceError::Unavailable
  })?;
  Ok(Provider(provider))
}

fn create_key(provider: &Provider) -> Result<Key, DeviceError> {
  let mut handle = NCRYPT_KEY_HANDLE::default();
  let name = HSTRING::from(KEY_NAME);
  let algorithm = HSTRING::from("RSA");
  unsafe {
    NCryptCreatePersistedKey(
      provider.0,
      &mut handle,
      PCWSTR(algorithm.as_ptr()),
      PCWSTR(name.as_ptr()),
      CERT_KEY_SPEC(0),
      NCRYPT_FLAGS(0),
    )
    .map_err(|err| {
      log_windows("create key", &err);
      DeviceError::Unavailable
    })?;
  }
  let key = Key(Some(handle));
  set_u32(&key, "Length", 2048)?;
  set_u32(&key, "Key Usage", NCRYPT_ALLOW_DECRYPT_FLAG)?;
  set_u32(&key, "Export Policy", 0)?;
  let raw = key.0.ok_or(DeviceError::Internal)?;
  unsafe { NCryptFinalizeKey(raw, NCRYPT_FLAGS(0)) }.map_err(|err| {
    log_windows("finalize key", &err);
    DeviceError::Unavailable
  })?;
  Ok(key)
}

fn set_u32(key: &Key, property: &str, value: u32) -> Result<(), DeviceError> {
  let handle = key.0.ok_or(DeviceError::Internal)?;
  let name = HSTRING::from(property);
  unsafe {
    NCryptSetProperty(handle.into(), PCWSTR(name.as_ptr()), &value.to_le_bytes(), NCRYPT_FLAGS(0)).map_err(|err| {
      log_windows("set property", &err);
      DeviceError::Internal
    })
  }
}

fn open_key(provider: &Provider, silent: bool) -> Result<Key, DeviceError> {
  let mut handle = NCRYPT_KEY_HANDLE::default();
  let name = HSTRING::from(KEY_NAME);
  let flags = if silent { NCRYPT_SILENT_FLAG } else { NCRYPT_FLAGS(0) };
  unsafe {
    NCryptOpenKey(provider.0, &mut handle, PCWSTR(name.as_ptr()), CERT_KEY_SPEC(0), flags).map_err(|err| {
      if is_missing_key(&err) {
        DeviceError::Invalid
      } else {
        log_windows("open key", &err);
        DeviceError::Internal
      }
    })?;
  }
  Ok(Key(Some(handle)))
}

fn delete_key(mut key: Key) -> Result<(), DeviceError> {
  let Some(handle) = key.0.take() else {
    return Ok(());
  };
  unsafe { NCryptDeleteKey(handle, 0) }.map_err(|err| {
    unsafe {
      let _ = NCryptFreeObject(handle.into());
    }
    log_windows("delete key", &err);
    DeviceError::Internal
  })
}

fn encrypt(key: &Key, dek: &[u8; 32]) -> Result<Vec<u8>, DeviceError> {
  let handle = key.0.ok_or(DeviceError::Internal)?;
  let padding = oaep_padding();
  let padding_ptr = ptr::from_ref(&padding).cast::<std::ffi::c_void>();
  let mut size = 0u32;
  unsafe {
    NCryptEncrypt(handle, Some(dek), Some(padding_ptr), None, &mut size, NCRYPT_PAD_OAEP_FLAG).map_err(|err| {
      log_windows("encrypt size", &err);
      DeviceError::Internal
    })?;
  }
  let mut ct = vec![0u8; size as usize];
  unsafe {
    NCryptEncrypt(
      handle,
      Some(dek),
      Some(padding_ptr),
      Some(&mut ct),
      &mut size,
      NCRYPT_PAD_OAEP_FLAG,
    )
    .map_err(|err| {
      log_windows("encrypt", &err);
      DeviceError::Internal
    })?;
  }
  ct.truncate(size as usize);
  if ct.is_empty() {
    return Err(DeviceError::Internal);
  }
  Ok(ct)
}

fn decrypt(key: &Key, ct: &[u8]) -> Result<Zeroizing<[u8; 32]>, DeviceError> {
  let handle = key.0.ok_or(DeviceError::Internal)?;
  let padding = oaep_padding();
  let padding_ptr = ptr::from_ref(&padding).cast::<std::ffi::c_void>();
  let mut size = 0u32;
  unsafe {
    NCryptDecrypt(handle, Some(ct), Some(padding_ptr), None, &mut size, NCRYPT_PAD_OAEP_FLAG).map_err(|err| {
      log_windows("decrypt size", &err);
      DeviceError::Invalid
    })?;
  }
  let mut plain = vec![0u8; size as usize];
  unsafe {
    NCryptDecrypt(
      handle,
      Some(ct),
      Some(padding_ptr),
      Some(&mut plain),
      &mut size,
      NCRYPT_PAD_OAEP_FLAG,
    )
    .map_err(|err| {
      log_windows("decrypt", &err);
      DeviceError::Invalid
    })?;
  }
  plain.truncate(size as usize);
  let dek: [u8; 32] = plain.as_slice().try_into().map_err(|_| DeviceError::Invalid)?;
  plain.fill(0);
  Ok(Zeroizing::new(dek))
}

fn oaep_padding() -> BCRYPT_OAEP_PADDING_INFO {
  BCRYPT_OAEP_PADDING_INFO { pszAlgId: BCRYPT_SHA256_ALGORITHM, pbLabel: ptr::null_mut(), cbLabel: 0 }
}

fn is_missing_key(err: &windows::core::Error) -> bool {
  let code = err.code().0 as u32;
  matches!(code, 0x8009_000D | 0x8009_0011 | 0x8009_0016)
}

fn log_windows(step: &str, err: &windows::core::Error) {
  tauri_plugin_log::log::error!("device slot windows {step}: {err}");
}
