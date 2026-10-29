/// Android 平台的启动挂钩
///
/// 编译到 Android 时被选为 `Platform` 的实现，并挂上设备槽插件。
///
/// # 示例
/// 见 [`Platform`](super::Platform)
pub(crate) struct Android;

#[cfg(target_os = "android")]
impl super::Platform for Android {
  /// 挂上设备槽插件
  ///
  /// # 参数
  /// - `builder`: 待挂插件的 Tauri 构建器
  ///
  /// # 返回值
  /// 挂上 `precession_device_slot` 后的构建器
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let builder = Android::plugins(builder);
  /// ```
  fn plugins(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.plugin(precession_device_slot::init())
  }

  /// 把 Android 设备槽接到数据库句柄
  ///
  /// 从应用状态取出 `DeviceApi`，交给 `AndroidDevice`。
  ///
  /// # 参数
  /// - `app`: 已管理 `DeviceApi` 的应用
  /// - `db`: 数据库句柄
  ///
  /// # 返回值
  /// 换上 `AndroidDevice` 后的句柄
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let db = Android::attach_device(app, db);
  /// ```
  fn attach_device(app: &tauri::App, db: crate::db::state::DbState) -> crate::db::state::DbState {
    use std::sync::Arc;

    use tauri::Manager;

    use crate::crypto::device::AndroidDevice;

    let api = app.state::<precession_device_slot::DeviceApi<tauri::Wry>>().inner().clone();
    db.with_device(Arc::new(AndroidDevice::new(api)))
  }
}

#[cfg(not(target_os = "android"))]
impl super::Platform for Android {}
