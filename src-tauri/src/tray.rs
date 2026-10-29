//! 系统托盘。桌面注册菜单；其他平台没有托盘，直接返回。

#![cfg_attr(not(desktop), allow(dead_code))]

/// 桌面：创建菜单并挂到托盘图标。
#[cfg(desktop)]
mod imp {
  use tauri::{
    App, AppHandle, Manager, WebviewWindow,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
  };

  /// 显示窗口并把它放到前台
  ///
  /// 窗口处于最小化时先取消最小化。显示、取消最小化和聚焦的错误被忽略。
  ///
  /// # 参数
  /// - `window`: 要放到前台的 WebView 窗口
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// focus_window(window);
  /// ```
  fn focus_window(window: WebviewWindow) {
    if window.is_minimized().unwrap_or(false) {
      let _ = window.unminimize();
    }
    let _ = window.show();
    let _ = window.set_focus();
  }

  /// 显示主窗口并把它放到前台
  ///
  /// 找不到名为 `main` 的窗口时不做任何事。窗口隐藏或最小化时先显示并取消最小化。
  ///
  /// # 参数
  /// - `app`: 应用句柄
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// focus_main_window(app);
  /// ```
  pub(crate) fn focus_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
      focus_window(window);
    }
  }

  /// 处理托盘菜单点击
  ///
  /// `quit` 以退出码 0 结束进程，`show` 聚焦主窗口，其他标识忽略。
  ///
  /// # 参数
  /// - `app`: 应用句柄
  /// - `event`: 菜单事件
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// handle_menu_event(app, event);
  /// ```
  fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
      "quit" => {
        app.exit(0);
      }
      "show" => {
        focus_main_window(app);
      }
      _ => {}
    }
  }

  /// 创建显示、分隔线和退出菜单，并挂到托盘图标
  ///
  /// 左键点击托盘时聚焦主窗口。菜单点击交给 `handle_menu_event`。
  ///
  /// # 参数
  /// - `app`: 正在启动的应用
  ///
  /// # 返回值
  /// 成功时不返回数据
  ///
  /// # 错误
  /// 创建菜单项、分隔线、菜单或托盘图标失败时返回该错误
  ///
  /// # Panics
  /// `default_window_icon()` 返回 `None` 时 `unwrap` 会 panic
  ///
  /// # 示例
  /// ```ignore
  /// register_tray_menu(app)?;
  /// ```
  pub fn register_tray_menu(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&show, &separator, &quit])?;

    let _tray = TrayIconBuilder::new()
      .icon(app.default_window_icon().unwrap().clone())
      .tooltip("Precession")
      .show_menu_on_left_click(false)
      .menu(&menu)
      .on_menu_event(|app, event| {
        handle_menu_event(app, event);
      })
      .on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
          focus_main_window(tray.app_handle());
        }
      })
      .build(app)?;
    Ok(())
  }
}

/// 没有托盘的平台直接成功返回。
#[cfg(not(desktop))]
mod imp {
  /// 在没有托盘的平台上直接成功返回
  ///
  /// # 参数
  /// - `app`: 应用句柄，此处不使用
  ///
  /// # 返回值
  /// 成功时不返回数据
  ///
  /// # 错误
  /// 不会返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// register_tray_menu(app)?;
  /// ```
  pub fn register_tray_menu(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let _ = app;
    Ok(())
  }
}

#[cfg(desktop)]
pub(crate) use imp::focus_main_window;
pub use imp::register_tray_menu;
