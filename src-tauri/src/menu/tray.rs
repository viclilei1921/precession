use tauri::{
  App, AppHandle, Manager,
  menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};

use crate::utils::window::focus_window;

// 这是一个纯 Rust 函数，处理事件
fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
  match event.id().as_ref() {
    "quit" => {
      app.exit(0);
    }
    "show" => {
      if let Some(window) = app.get_webview_window("main") {
        focus_window(window);
      }
    }
    _ => {}
  }
}

pub fn register_tray_menu(app: &App) -> Result<(), Box<dyn std::error::Error>> {
  // 1. 创建菜单项
  // 参数: manager, id, text, enabled, accelerator
  let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
  let show_i = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
  let separator = PredefinedMenuItem::separator(app).unwrap();

  // 2. 创建菜单
  let menu = Menu::with_items(app, &[&show_i, &separator, &quit_i])?;

  let _tray = TrayIconBuilder::new()
    .icon(app.default_window_icon().unwrap().clone())
    .tooltip("Precession")
    .show_menu_on_left_click(false)
    .menu(&menu)
    // 4. 处理菜单点击事件 (右键菜单)
    .on_menu_event(|app, event| {
      handle_menu_event(app, event);
    })
    // 5. 处理托盘图标本身的点击事件 (左键点击)
    .on_tray_icon_event(|tray, event| {
      if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
        let app = tray.app_handle();
        if let Some(window) = app.get_webview_window("main") {
          focus_window(window);
        }
      }
    })
    .build(app)?;
  Ok(())
}
