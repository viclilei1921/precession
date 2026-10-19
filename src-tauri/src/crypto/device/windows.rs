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
  /// 用 TPM 上的 RSA 密钥包住 DEK，再收成 `device.wrap`
  ///
  /// # 参数
  /// - `dek`: 32 字节 DEK
  ///
  /// # 返回值
  /// 种类为 `windows-cng` 的包裹字节
  ///
  /// # 错误
  /// TPM 封装失败时返回该 `DeviceError`；编码 JSON 失败时返回 `Internal`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let blob = device.enroll(dek)?;
  /// ```
  fn enroll(&self, dek: &[u8; 32]) -> Result<Vec<u8>, DeviceError> {
    let ct = seal_with_tpm(dek)?;
    wrap::encode(KIND_WINDOWS, &ct)
  }

  /// 从 Windows 包裹里取出密文并用 TPM 解开
  ///
  /// 种类必须是 `windows-cng`。
  ///
  /// # 参数
  /// - `blob`: `device.wrap` 的全部字节
  ///
  /// # 返回值
  /// 成功时返回会在丢弃时清零的 DEK
  ///
  /// # 错误
  /// 包裹无法解析或种类不符时返回 `Invalid`；TPM 解密失败时返回该 `DeviceError`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let dek = device.open(blob)?;
  /// ```
  fn open(&self, blob: &[u8]) -> Result<Zeroizing<[u8; 32]>, DeviceError> {
    let parsed = wrap::decode(blob)?;
    if parsed.kind != KIND_WINDOWS {
      return Err(DeviceError::Invalid);
    }
    open_with_tpm(&parsed.ct)
  }

  /// 弹出 Windows Hello 做用户验证
  ///
  /// # 返回值
  /// 成功时不返回数据
  ///
  /// # 错误
  /// 验证失败时返回 `verify_hello` 给出的 `DeviceError`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// device.confirm()?;
  /// ```
  fn confirm(&self) -> Result<(), DeviceError> {
    verify_hello()
  }

  /// 静默打开并删除 TPM 上的设备密钥
  ///
  /// 密钥已经不存在时视为成功。
  ///
  /// # 返回值
  /// 成功时不返回数据
  ///
  /// # 错误
  /// 打开提供程序失败，或密钥存在但删除失败时返回对应的 `DeviceError`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// device.forget()?;
  /// ```
  fn forget(&self) -> Result<(), DeviceError> {
    let provider = open_provider()?;
    match open_key(&provider, true) {
      Ok(key) => delete_key(key),
      Err(DeviceError::Invalid) => Ok(()),
      Err(err) => Err(err),
    }
  }
}

/// 检查 Windows Hello 是否可用，并向当前前台窗口请求验证
///
/// # 返回值
/// 结果为 `Verified` 时不返回数据
///
/// # 错误
/// - 可用性检查、互操作或发起请求失败，前台窗口为空，或可用性不是 `Available` 时返回 `Unavailable`
/// - 等待结果失败时返回 `Internal`
/// - 用户取消时返回 `Cancelled`
/// - 其他验证结果返回 `Unavailable`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// verify_hello()?;
/// ```
fn verify_hello() -> Result<(), DeviceError> {
  let availability = UserConsentVerifier::CheckAvailabilityAsync()
    .map_err(|err| {
      log_windows("hello availability", &err);
      DeviceError::Unavailable
    })?
    .join()
    .map_err(|err| {
      log_windows("hello availability join", &err);
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
  let result = operation.join().map_err(|err| {
    log_windows("hello join", &err);
    DeviceError::Internal
  })?;
  match result {
    UserConsentVerificationResult::Verified => Ok(()),
    UserConsentVerificationResult::Canceled => Err(DeviceError::Cancelled),
    _ => Err(DeviceError::Unavailable),
  }
}

/// 删掉已有 TPM 密钥后新建一把，并加密 DEK
///
/// 加密失败时尝试删掉刚建的密钥。
///
/// # 参数
/// - `dek`: 32 字节 DEK
///
/// # 返回值
/// TPM 加密后的密文
///
/// # 错误
/// 打开提供程序、删除旧密钥、创建密钥或加密失败时返回对应的 `DeviceError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let ct = seal_with_tpm(dek)?;
/// ```
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

/// 静默打开 TPM 密钥并解密
///
/// # 参数
/// - `ct`: TPM 密文
///
/// # 返回值
/// 成功时返回会在丢弃时清零的 DEK
///
/// # 错误
/// 打开提供程序、打开密钥或解密失败时返回对应的 `DeviceError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let dek = open_with_tpm(ct)?;
/// ```
fn open_with_tpm(ct: &[u8]) -> Result<Zeroizing<[u8; 32]>, DeviceError> {
  let provider = open_provider()?;
  let key = open_key(&provider, true)?;
  decrypt(&key, ct)
}

struct Provider(NCRYPT_PROV_HANDLE);

impl Drop for Provider {
  /// 释放仍然有效的 CNG 提供程序句柄
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// drop(provider);
  /// ```
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
  /// 释放仍然持有的 CNG 密钥句柄
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// drop(key);
  /// ```
  fn drop(&mut self) {
    if let Some(handle) = self.0.take() {
      unsafe {
        let _ = NCryptFreeObject(handle.into());
      }
    }
  }
}

/// 打开 Microsoft Platform Crypto Provider
///
/// # 返回值
/// 成功时返回提供程序句柄
///
/// # 错误
/// 打开失败时返回 `Unavailable`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let provider = open_provider()?;
/// ```
fn open_provider() -> Result<Provider, DeviceError> {
  let mut provider = NCRYPT_PROV_HANDLE::default();
  let name = HSTRING::from(PROVIDER_NAME);
  unsafe { NCryptOpenStorageProvider(&mut provider, PCWSTR(name.as_ptr()), 0) }.map_err(|err| {
    log_windows("open provider", &err);
    DeviceError::Unavailable
  })?;
  Ok(Provider(provider))
}

/// 在 TPM 上创建不可导出的 2048 位 RSA 密钥
///
/// 密钥用途只有解密，导出策略为 0，最后固化密钥。
///
/// # 参数
/// - `provider`: 已打开的平台加密提供程序
///
/// # 返回值
/// 固化后的密钥句柄
///
/// # 错误
/// - 创建或固化失败时返回 `Unavailable`
/// - 句柄在设置属性后缺失时返回 `Internal`
/// - 设置属性失败时返回 `set_u32` 给出的 `DeviceError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let key = create_key(&provider)?;
/// ```
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

/// 给 CNG 密钥设置一个 `u32` 属性
///
/// # 参数
/// - `key`: 密钥句柄
/// - `property`: 属性名
/// - `value`: 小端写入的数值
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 句柄缺失时返回 `Internal`；`NCryptSetProperty` 失败时返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// set_u32(&key, "Length", 2048)?;
/// ```
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

/// 按名字打开 TPM 上的设备密钥
///
/// # 参数
/// - `provider`: 已打开的平台加密提供程序
/// - `silent`: 为真时带 `NCRYPT_SILENT_FLAG`
///
/// # 返回值
/// 打开的密钥句柄
///
/// # 错误
/// 密钥不存在时返回 `Invalid`；其他打开失败返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let key = open_key(&provider, true)?;
/// ```
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

/// 删除一把 CNG 密钥
///
/// 句柄已经空着时直接成功。删除失败时仍释放句柄。
///
/// # 参数
/// - `key`: 要删除的密钥
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// `NCryptDeleteKey` 失败时返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// delete_key(key)?;
/// ```
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

/// 用 OAEP-SHA256 加密 DEK
///
/// # 参数
/// - `key`: TPM 密钥
/// - `dek`: 32 字节 DEK
///
/// # 返回值
/// 截到实际长度后的密文
///
/// # 错误
/// 句柄缺失、加密失败或密文为空时返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let ct = encrypt(&key, dek)?;
/// ```
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

/// 用 OAEP-SHA256 解密，并要求明文正好 32 字节
///
/// 拷出 DEK 后把临时缓冲填 0。
///
/// # 参数
/// - `key`: TPM 密钥
/// - `ct`: 密文
///
/// # 返回值
/// 成功时返回会在丢弃时清零的 DEK
///
/// # 错误
/// 句柄缺失时返回 `Internal`；解密失败或明文不是 32 字节时返回 `Invalid`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let dek = decrypt(&key, ct)?;
/// ```
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

/// 返回 SHA-256、无标签的 OAEP 填充信息
///
/// # 返回值
/// `pszAlgId` 为 SHA-256、标签指针为空、标签长度为 0 的结构
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let info = oaep_padding();
/// assert_eq!(info.cbLabel, 0);
/// assert!(info.pbLabel.is_null());
/// ```
fn oaep_padding() -> BCRYPT_OAEP_PADDING_INFO {
  BCRYPT_OAEP_PADDING_INFO { pszAlgId: BCRYPT_SHA256_ALGORITHM, pbLabel: ptr::null_mut(), cbLabel: 0 }
}

/// 判断错误码是不是找不到 CNG 密钥
///
/// # 参数
/// - `err`: Windows 错误
///
/// # 返回值
/// 代码为 `0x8009000D`、`0x80090011` 或 `0x80090016` 时为 true
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// use windows::core::{Error, HRESULT};
/// let err = Error::from_hresult(HRESULT(0x8009_000Du32 as i32));
/// assert!(is_missing_key(&err));
/// ```
fn is_missing_key(err: &windows::core::Error) -> bool {
  let code = err.code().0 as u32;
  matches!(code, 0x8009_000D | 0x8009_0011 | 0x8009_0016)
}

/// 记下 Windows 设备槽某一步的错误
///
/// # 参数
/// - `step`: 失败步骤名
/// - `err`: Windows 错误
///
/// # 返回值
/// 不返回数据
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// log_windows("open provider", err);
/// ```
fn log_windows(step: &str, err: &windows::core::Error) {
  tauri_plugin_log::log::error!("device slot windows {step}: {err}");
}
