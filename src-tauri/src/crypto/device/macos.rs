//! 钥匙串保存随机设备 KEK。用户已登录时直接读取。
//! 用户锁定之后再解锁时，先要求 Touch ID。只属于这台设备。

use block2::RcBlock;
use objc2::runtime::Bool;
use objc2_foundation::{NSError, NSString};
use objc2_local_authentication::{LAContext, LAPolicy};
use security_framework::access_control::{ProtectionMode, SecAccessControl};
use security_framework::passwords::{
  PasswordOptions, delete_generic_password_options, generic_password, set_generic_password_options,
};
use security_framework_sys::base::{
  errSecAuthFailed, errSecInteractionNotAllowed, errSecItemNotFound, errSecMissingEntitlement, errSecNotAvailable,
  errSecUserCanceled,
};
use zeroize::{Zeroize, Zeroizing};

use super::error::DeviceError;
use super::wrap;

const KIND_MACOS: &str = "macos-keychain";
use crate::crypto::constants::NONCE_LEN;
use crate::crypto::dek::{random_dek, unwrap_dek, wrap_dek};

const SERVICE: &str = "precession";
const ACCOUNT: &str = "device-kek-v2";

pub(crate) struct MacosDevice;

impl super::DeviceSlot for MacosDevice {
  fn enroll(&self, dek: &[u8; 32]) -> Result<Vec<u8>, DeviceError> {
    let kek = random_dek().map_err(|_| DeviceError::Internal)?;
    if let Err(err) = store_kek(kek.as_ref()) {
      let _ = delete_kek();
      return Err(err);
    }
    let (nonce, ct) = wrap_dek(&kek, dek).map_err(|_| DeviceError::Internal)?;
    let mut packed = Vec::with_capacity(NONCE_LEN + ct.len());
    packed.extend_from_slice(&nonce);
    packed.extend_from_slice(&ct);
    wrap::encode(KIND_MACOS, &packed)
  }

  fn open(&self, blob: &[u8]) -> Result<Zeroizing<[u8; 32]>, DeviceError> {
    let parsed = wrap::decode(blob)?;
    if parsed.kind != KIND_MACOS || parsed.ct.len() <= NONCE_LEN {
      return Err(DeviceError::Invalid);
    }
    let kek = read_kek()?;
    let (nonce, ct) = parsed.ct.split_at(NONCE_LEN);
    unwrap_dek(&kek, nonce, ct).map_err(|_| DeviceError::Invalid)
  }

  fn confirm(&self) -> Result<(), DeviceError> {
    confirm_touch_id()
  }

  fn forget(&self) -> Result<(), DeviceError> {
    delete_kek()
  }
}

fn confirm_touch_id() -> Result<(), DeviceError> {
  let context = unsafe { LAContext::new() };
  let policy = LAPolicy::DeviceOwnerAuthenticationWithBiometrics;
  if let Err(err) = unsafe { context.canEvaluatePolicy_error(policy) } {
    tauri_plugin_log::log::error!("device slot macos touch id unavailable: {}", err.code());
    return Err(DeviceError::Unavailable);
  }

  let (tx, rx) = std::sync::mpsc::channel();
  let keepalive = context.clone();
  let reply = RcBlock::new(move |success: Bool, error: *mut NSError| {
    let _keepalive = &keepalive;
    let outcome = if bool::from(success) {
      Ok(())
    } else if error.is_null() {
      Err(DeviceError::Unavailable)
    } else {
      Err(map_touch_id(unsafe { &*error }.code()))
    };
    let _ = tx.send(outcome);
  });
  let reason = NSString::from_str("解锁人生档案");
  unsafe { context.evaluatePolicy_localizedReason_reply(policy, &reason, &reply) };
  rx.recv().unwrap_or(Err(DeviceError::Internal))
}

fn map_touch_id(code: objc2_foundation::NSInteger) -> DeviceError {
  // LAError: userCancel -2, userFallback -3, systemCancel -4, appCancel -9。
  match code {
    -2 | -3 | -4 | -9 => DeviceError::Cancelled,
    _ => {
      tauri_plugin_log::log::error!("device slot macos touch id failed: {code}");
      DeviceError::Unavailable
    }
  }
}

fn base_options() -> PasswordOptions {
  let mut options = PasswordOptions::new_generic_password(SERVICE, ACCOUNT);
  options.use_protected_keychain();
  options.set_access_synchronized(Some(false));
  options
}

fn store_kek(kek: &[u8; 32]) -> Result<(), DeviceError> {
  delete_kek()?;
  let access = SecAccessControl::create_with_protection(Some(ProtectionMode::AccessibleWhenUnlockedThisDeviceOnly), 0)
    .map_err(|err| {
      tauri_plugin_log::log::error!("device slot macos access control: {err}");
      DeviceError::Unavailable
    })?;
  let mut options = base_options();
  options.set_access_control(access);
  options.set_label("Precession 设备密钥");
  set_generic_password_options(kek, options).map_err(map_sec)
}

fn read_kek() -> Result<Zeroizing<[u8; 32]>, DeviceError> {
  let mut bytes = generic_password(base_options()).map_err(map_sec)?;
  if bytes.len() != 32 {
    bytes.zeroize();
    return Err(DeviceError::Invalid);
  }
  let mut kek = Zeroizing::new([0u8; 32]);
  kek.copy_from_slice(&bytes);
  bytes.zeroize();
  Ok(kek)
}

fn delete_kek() -> Result<(), DeviceError> {
  match delete_generic_password_options(base_options()) {
    Ok(()) => Ok(()),
    Err(err) if err.code() == errSecItemNotFound => Ok(()),
    Err(err) => Err(map_sec(err)),
  }
}

fn map_sec(err: security_framework::base::Error) -> DeviceError {
  let code = err.code();
  if code == errSecUserCanceled || code == errSecAuthFailed {
    return DeviceError::Cancelled;
  }
  if code == errSecItemNotFound {
    return DeviceError::Invalid;
  }
  if code == errSecInteractionNotAllowed || code == errSecMissingEntitlement || code == errSecNotAvailable {
    tauri_plugin_log::log::error!("device slot macos unavailable: {err}");
    return DeviceError::Unavailable;
  }
  tauri_plugin_log::log::error!("device slot macos: {err}");
  DeviceError::Internal
}
