/// Windows 平台的启动挂钩
///
/// 编译到 Windows 时被选为 `Platform` 的实现，并挂上 shell 和对话框插件。
///
/// # 示例
/// 见 [`Platform`](super::Platform)
pub(crate) struct Windows;

#[cfg(target_os = "windows")]
impl super::Platform for Windows {
  /// 挂上 shell 和对话框插件
  ///
  /// # 参数
  /// - `builder`: 待挂插件的 Tauri 构建器
  ///
  /// # 返回值
  /// 挂上两个插件后的构建器
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let builder = Windows::plugins(builder);
  /// ```
  fn plugins(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    // 初始化 shell 插件(为了侧车功能)。
    builder.plugin(tauri_plugin_shell::init()).plugin(tauri_plugin_dialog::init())
  }

  /// 登记侧车进程槽并注册托盘菜单
  ///
  /// 托盘注册失败时只记错误日志。
  ///
  /// # 参数
  /// - `app`: 正在启动的应用
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// Windows::setup(app);
  /// ```
  fn setup(app: &mut tauri::App) {
    use tauri::Manager;

    // 管理侧车进程，并注册托盘菜单。
    app.manage(crate::sidecar::ProcessSlot::default());
    if let Err(e) = crate::tray::register_tray_menu(app) {
      tauri_plugin_log::log::error!("tray menu registration failed: {e}");
    }
  }

  /// 把关闭请求改成隐藏窗口
  ///
  /// # 参数
  /// - `window`: 发出事件的窗口
  /// - `event`: 窗口事件
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// Windows::on_window_event(window, event);
  /// ```
  fn on_window_event(window: &tauri::Window, event: &tauri::WindowEvent) {
    // 关闭窗口时隐藏到托盘。
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
      api.prevent_close();
      let _ = window.hide();
    }
  }
}

#[cfg(not(target_os = "windows"))]
impl super::Platform for Windows {}
