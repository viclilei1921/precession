//! Android 设备槽走应用内插件。DEK 只在进程内交给 Keystore，不进 WebView。

use zeroize::Zeroizing;

use super::error::DeviceError;
use super::wrap;

const KIND_ANDROID: &str = "android-keystore";
use precession_device_slot::{DeviceApi, SlotError};

pub(crate) struct AndroidDevice {
  api: DeviceApi<tauri::Wry>,
}

impl AndroidDevice {
  /// 用插件句柄创建 Android 设备槽
  ///
  /// # 参数
  /// - `api`: 设备槽插件接口
  ///
  /// # 返回值
  /// 持有该接口的 `AndroidDevice`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let device = AndroidDevice::new(api);
  /// ```
  pub(crate) fn new(api: DeviceApi<tauri::Wry>) -> Self {
    Self { api }
  }
}

impl super::DeviceSlot for AndroidDevice {
  /// 让 Keystore 包装 DEK，再收成 `device.wrap`
  ///
  /// # 参数
  /// - `dek`: 32 字节 DEK
  ///
  /// # 返回值
  /// 种类为 `android-keystore` 的包裹字节
  ///
  /// # 错误
  /// 插件调用失败时按 `map_slot` 转换；编码 JSON 失败时返回 `Internal`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let blob = device.enroll(dek)?;
  /// ```
  fn enroll(&self, dek: &[u8; 32]) -> Result<Vec<u8>, DeviceError> {
    let ct = self.api.enroll(dek).map_err(map_slot)?;
    wrap::encode(KIND_ANDROID, &ct)
  }

  /// 从 Android 包裹里解开 DEK
  ///
  /// 种类必须是 `android-keystore`，解开的字节必须是 32 字节。
  ///
  /// # 参数
  /// - `blob`: `device.wrap` 的全部字节
  ///
  /// # 返回值
  /// 成功时返回会在丢弃时清零的 DEK
  ///
  /// # 错误
  /// 包裹无法解析、种类不符或长度不是 32 字节时返回 `Invalid`；插件失败时按 `map_slot` 转换
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
    if parsed.kind != KIND_ANDROID {
      return Err(DeviceError::Invalid);
    }
    let dek = self.api.open(&parsed.ct).map_err(map_slot)?;
    let dek: [u8; 32] = dek.as_slice().try_into().map_err(|_| DeviceError::Invalid)?;
    Ok(Zeroizing::new(dek))
  }

  /// 调用插件做系统验证
  ///
  /// # 返回值
  /// 成功时不返回数据
  ///
  /// # 错误
  /// 插件失败时按 `map_slot` 转换成 `DeviceError`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// device.confirm()?;
  /// ```
  fn confirm(&self) -> Result<(), DeviceError> {
    self.api.confirm().map_err(map_slot)
  }

  /// 调用插件删除 Keystore 里的设备密钥
  ///
  /// # 返回值
  /// 成功时不返回数据
  ///
  /// # 错误
  /// 插件失败时按 `map_slot` 转换成 `DeviceError`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// device.forget()?;
  /// ```
  fn forget(&self) -> Result<(), DeviceError> {
    self.api.forget().map_err(map_slot)
  }
}

/// 把插件错误码映射成设备槽错误
///
/// # 参数
/// - `err`: 插件返回的 `SlotError`
///
/// # 返回值
/// 同名含义的 `DeviceError` 变体
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(matches!(map_slot(SlotError::Invalid), DeviceError::Invalid));
/// assert!(matches!(map_slot(SlotError::Cancelled), DeviceError::Cancelled));
/// ```
fn map_slot(err: SlotError) -> DeviceError {
  match err {
    SlotError::Unavailable => DeviceError::Unavailable,
    SlotError::Cancelled => DeviceError::Cancelled,
    SlotError::Invalid => DeviceError::Invalid,
    SlotError::Internal => DeviceError::Internal,
  }
}
