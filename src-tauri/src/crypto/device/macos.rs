//! 钥匙串保存随机设备 KEK。用户已登录时直接读取。
//! 用户锁定之后再解锁时，先要求 Touch ID。只属于这台设备。

use block2::RcBlock;
use objc2::runtime::Bool;
use objc2_foundation::{NSError, NSString};
use objc2_local_authentication::{LAContext, LAPolicy};
use objc2_security::{
  errSecAuthFailed, errSecInteractionNotAllowed, errSecItemNotFound, errSecMissingEntitlement, errSecNotAvailable,
  errSecUserCanceled,
};
use security_framework::access_control::{ProtectionMode, SecAccessControl};
use security_framework::passwords::{
  PasswordOptions, delete_generic_password_options, generic_password, set_generic_password_options,
};
use zeroize::{Zeroize, Zeroizing};

use super::error::DeviceError;
use super::wrap;

const KIND_MACOS: &str = "macos-keychain";
use crate::crypto::constants::NONCE_LEN;
use crate::crypto::dek::{random_dek, unwrap_dek, wrap_dek};

const SERVICE: &str = "precession";
const ACCOUNT: &str = "device-kek-v2";

/// macOS 设备槽
///
/// 钥匙串保存随机设备 KEK。用户已登录时直接读取。用户锁定之后再解锁时，先要求 Touch ID。只属于这台设备。
///
/// # 示例
/// ```ignore
/// let blob = super::wrap::encode("other", b"not-empty")?;
/// assert!(matches!(MacosDevice.open(&blob), Err(DeviceError::Invalid)));
/// ```
pub(crate) struct MacosDevice;

impl super::DeviceSlot for MacosDevice {
  /// 生成设备 KEK，写入钥匙串，再包住 DEK
  ///
  /// 写入钥匙串失败时会尝试删掉已写入的项。
  ///
  /// # 参数
  /// - `dek`: 32 字节 DEK
  ///
  /// # 返回值
  /// 种类为 `macos-keychain` 的包裹字节
  ///
  /// # 错误
  /// 生成 KEK、写入钥匙串、包装或编码失败时返回对应的 `DeviceError`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let blob = device.enroll(dek)?;
  /// ```
  fn enroll(&self, dek: &[u8; 32]) -> Result<Vec<u8>, DeviceError> {
    let kek = random_dek().map_err(|_| DeviceError::Internal)?;
    if let Err(err) = store_kek(&kek) {
      let _ = delete_kek();
      return Err(err);
    }
    let (nonce, ct) = wrap_dek(&kek, dek).map_err(|_| DeviceError::Internal)?;
    let mut packed = Vec::with_capacity(NONCE_LEN + ct.len());
    packed.extend_from_slice(&nonce);
    packed.extend_from_slice(&ct);
    wrap::encode(KIND_MACOS, &packed)
  }

  /// 从钥匙串读出 KEK 并解开包裹
  ///
  /// 种类必须是 `macos-keychain`，密文必须长于 nonce。
  ///
  /// # 参数
  /// - `blob`: `device.wrap` 的全部字节
  ///
  /// # 返回值
  /// 成功时返回会在丢弃时清零的 DEK
  ///
  /// # 错误
  /// 包裹不合法或解密失败时返回 `Invalid`；读钥匙串失败时返回对应的 `DeviceError`
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
    if parsed.kind != KIND_MACOS || parsed.ct.len() <= NONCE_LEN {
      return Err(DeviceError::Invalid);
    }
    let kek = read_kek()?;
    let (nonce, ct) = parsed.ct.split_at(NONCE_LEN);
    unwrap_dek(&kek, nonce, ct).map_err(|_| DeviceError::Invalid)
  }

  /// 要求 Touch ID 验证
  ///
  /// # 返回值
  /// 成功时不返回数据
  ///
  /// # 错误
  /// 验证失败时返回 `confirm_touch_id` 给出的 `DeviceError`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// device.confirm()?;
  /// ```
  fn confirm(&self) -> Result<(), DeviceError> {
    confirm_touch_id()
  }

  /// 删除数据保护和登录钥匙串里的设备 KEK
  ///
  /// # 返回值
  /// 成功时不返回数据
  ///
  /// # 错误
  /// 删除失败时返回 `delete_kek` 给出的 `DeviceError`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// device.forget()?;
  /// ```
  fn forget(&self) -> Result<(), DeviceError> {
    delete_kek()
  }
}

/// 弹出 Touch ID，并等待验证回调
///
/// 设备不能做生物识别时返回 `Unavailable`。回调没送到结果时返回 `Internal`。
///
/// # 返回值
/// 验证成功时不返回数据
///
/// # 错误
/// - 不能评估策略时返回 `Unavailable`
/// - 回调报告失败时返回 `map_touch_id` 的结果；错误指针为空时返回 `Unavailable`
/// - 通道收不到结果时返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// confirm_touch_id()?;
/// ```
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

/// 把 Touch ID 错误码映射成设备槽错误
///
/// # 参数
/// - `code`: `LAError` 代码
///
/// # 返回值
/// `-2`、`-3`、`-4`、`-9` 返回 `Cancelled`，其余记日志并返回 `Unavailable`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let err = map_touch_id(-2);
/// ```
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

/// 组装钥匙串通用密码的查询选项
///
/// `protected` 为真时使用受保护钥匙串。同步始终关掉。
///
/// # 参数
/// - `protected`: 是否使用数据保护钥匙串
///
/// # 返回值
/// 服务名为 `precession`、账户名为 `device-kek-v2` 的选项
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let options = base_options(true);
/// ```
fn base_options(protected: bool) -> PasswordOptions {
  let mut options = PasswordOptions::new_generic_password(SERVICE, ACCOUNT);
  if protected {
    options.use_protected_keychain();
  }
  options.set_access_synchronized(Some(false));
  options
}

/// 先删掉旧项，再把 KEK 写入钥匙串
///
/// 受保护钥匙串因缺少授权失败时，改写入登录钥匙串。
///
/// # 参数
/// - `kek`: 32 字节设备 KEK
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 删除旧项失败，或两次写入都失败时返回对应的 `DeviceError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// store_kek(kek)?;
/// ```
fn store_kek(kek: &[u8; 32]) -> Result<(), DeviceError> {
  delete_kek()?;
  match store_into(kek, true) {
    Ok(()) => Ok(()),
    Err(err) if err.code() == errSecMissingEntitlement => {
      tauri_plugin_log::log::warn!(
        "device slot macos: data protection keychain needs a signed entitlement, using login keychain"
      );
      store_into(kek, false).map_err(map_sec)
    }
    Err(err) => Err(map_sec(err)),
  }
}

/// 按是否受保护，把 KEK 写成通用密码
///
/// # 参数
/// - `kek`: 32 字节设备 KEK
/// - `protected`: 为真时附上仅本机解锁可访问的访问控制
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 创建访问控制或写入钥匙串失败时返回 `security_framework::base::Error`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// store_into(kek, true)?;
/// ```
fn store_into(kek: &[u8; 32], protected: bool) -> Result<(), security_framework::base::Error> {
  let mut options = base_options(protected);
  if protected {
    let access =
      SecAccessControl::create_with_protection(Some(ProtectionMode::AccessibleWhenUnlockedThisDeviceOnly), 0).map_err(
        |err| {
          tauri_plugin_log::log::error!("device slot macos access control: {err}");
          err
        },
      )?;
    options.set_access_control(access);
  }
  options.set_label("Precession 设备密钥");
  set_generic_password_options(kek, options)
}

/// 从受保护钥匙串读取 KEK，失败时再试登录钥匙串
///
/// 缺少授权或找不到项时才回退；其他错误直接转换后返回。
///
/// # 返回值
/// 成功时返回会在丢弃时清零的 32 字节 KEK
///
/// # 错误
/// 两处都读失败，或字节长度不是 32 时返回对应的 `DeviceError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let kek = read_kek()?;
/// ```
fn read_kek() -> Result<Zeroizing<[u8; 32]>, DeviceError> {
  match load_bytes(true) {
    Ok(bytes) => kek_from_bytes(bytes),
    Err(err) if err.code() == errSecMissingEntitlement || err.code() == errSecItemNotFound => {
      kek_from_bytes(load_bytes(false).map_err(map_sec)?)
    }
    Err(err) => Err(map_sec(err)),
  }
}

/// 按选项从钥匙串读出通用密码字节
///
/// # 参数
/// - `protected`: 是否读取受保护钥匙串
///
/// # 返回值
/// 读到的字节
///
/// # 错误
/// 钥匙串查询失败时返回 `security_framework::base::Error`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let bytes = load_bytes(false)?;
/// ```
fn load_bytes(protected: bool) -> Result<Vec<u8>, security_framework::base::Error> {
  generic_password(base_options(protected))
}

/// 把钥匙串字节收成 32 字节 KEK，并清掉原缓冲
///
/// # 参数
/// - `bytes`: 钥匙串里读出的字节
///
/// # 返回值
/// 成功时返回会在丢弃时清零的 KEK
///
/// # 错误
/// 长度不是 32 时先清掉缓冲，再返回 `Invalid`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(kek_from_bytes(vec![1, 2, 3]).is_err());
/// assert!(kek_from_bytes(vec![0; 32]).is_ok());
/// ```
fn kek_from_bytes(mut bytes: Vec<u8>) -> Result<Zeroizing<[u8; 32]>, DeviceError> {
  if bytes.len() != 32 {
    bytes.zeroize();
    return Err(DeviceError::Invalid);
  }
  let mut kek = Zeroizing::new([0u8; 32]);
  kek.copy_from_slice(&bytes);
  bytes.zeroize();
  Ok(kek)
}

/// 删除受保护钥匙串和登录钥匙串里的设备 KEK
///
/// # 返回值
/// 两处都处理完时不返回数据
///
/// # 错误
/// 任一次删除返回错误时传出该 `DeviceError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// delete_kek()?;
/// ```
fn delete_kek() -> Result<(), DeviceError> {
  delete_one(true)?;
  delete_one(false)
}

/// 删除一处钥匙串里的设备 KEK
///
/// 项不存在时视为成功。受保护钥匙串缺少授权时也视为成功。
///
/// # 参数
/// - `protected`: 是否删除受保护钥匙串中的项
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 其他钥匙串错误经 `map_sec` 转换后返回
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// delete_one(false)?;
/// ```
fn delete_one(protected: bool) -> Result<(), DeviceError> {
  match delete_generic_password_options(base_options(protected)) {
    Ok(()) => Ok(()),
    Err(err) if err.code() == errSecItemNotFound => Ok(()),
    Err(err) if protected && err.code() == errSecMissingEntitlement => Ok(()),
    Err(err) => Err(map_sec(err)),
  }
}

/// 把钥匙串状态码映射成设备槽错误
///
/// # 参数
/// - `err`: Security 框架错误
///
/// # 返回值
/// 用户取消或认证失败为 `Cancelled`；找不到项为 `Invalid`；不允许交互、缺少授权或不可用为 `Unavailable`；其余为 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let mapped = map_sec(err);
/// ```
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
