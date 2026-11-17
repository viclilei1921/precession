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
    // 触摸板手指接触监听：失败只记日志，前端走静默结算兜底
    super::windows_raw::attach(app.handle());
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
    // 窗口真正销毁前摘掉触摸板子类过程
    if matches!(event, tauri::WindowEvent::Destroyed) {
      super::windows_raw::detach();
    }
  }

  /// 允许触摸板捏合变成页面里的滚轮事件
  ///
  /// WebView2 默认关掉手势缩放，捏合到不了页面。打开之后和浏览器一样，是带 Ctrl 的滚轮。
  /// 页面会拦住默认行为，避免整个窗口被放大。
  ///
  /// # 参数
  /// - `webview`: 加载完成的 WebView
  /// - `payload`: 页面加载事件
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// Windows::on_page_load(webview, payload);
  /// ```
  fn on_page_load(webview: &tauri::Webview, payload: &tauri::webview::PageLoadPayload<'_>) {
    if payload.event() != tauri::webview::PageLoadEvent::Finished {
      return;
    }
    allow_pinch_gesture(webview);
  }
}

/// 放行 WebView2 的捏合手势，让触摸板/触摸屏捏合能作为带 Ctrl 的滚轮事件到达页面
///
/// wry 在创建 WebView 时把 `IsZoomControlEnabled` 和 `IsPinchZoomEnabled` 都绑在
/// `zoomHotkeysEnabled` 上（两个都是导航级设置，中途改动要等下次导航），所以
/// tauri.conf.json 里必须保持 true——否则捏合在首次加载就被关死，运行时救不回来。
/// 页面 wheel 捕获阶段 preventDefault，浏览器级缩放实际到不了页面。
/// 这里每次页面加载幂等重申 IsPinchZoomEnabled，兜住配置缺失的情形。
#[cfg(target_os = "windows")]
fn allow_pinch_gesture(webview: &tauri::Webview) {
  use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings5;
  use windows::core::Interface;

  let scheduled = webview.with_webview(|inner| {
    let enabled = (|| {
      let core = unsafe { inner.controller().CoreWebView2() }?;
      let settings = unsafe { core.Settings() }?;
      let settings5 = settings.cast::<ICoreWebView2Settings5>()?;
      // 捏合手势（触摸板双指 / 触摸屏双指）：放行给页面处理。
      // 浏览器级整页缩放由页面 preventDefault 拦截，不会触发。
      unsafe { settings5.SetIsPinchZoomEnabled(true) }?;
      Ok::<(), windows::core::Error>(())
    })();
    if let Err(error) = enabled {
      tauri_plugin_log::log::error!("allow pinch gesture failed: {error}");
    }
  });
  if let Err(error) = scheduled {
    tauri_plugin_log::log::error!("allow pinch gesture failed: {error}");
  }
}

#[cfg(not(target_os = "windows"))]
impl super::Platform for Windows {}
