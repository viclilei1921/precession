use tauri::{
  App, AppHandle, Manager, WebviewWindow,
  menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};

fn focus_window(window: WebviewWindow) {
  if window.is_minimized().unwrap_or(false) {
    let _ = window.unminimize();
  }
  let _ = window.show();
  let _ = window.set_focus();
}

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
        let app = tray.app_handle();
        if let Some(window) = app.get_webview_window("main") {
          focus_window(window);
        }
      }
    })
    .build(app)?;
  Ok(())
}
