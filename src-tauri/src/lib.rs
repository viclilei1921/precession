mod constants;
mod crypto;
mod db;
mod db_demo;
#[cfg(desktop)]
mod menu;
mod plugin;
mod utils;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let builder = tauri::Builder::default();

  // 初始化日志
  let builder = builder.plugin(plugin::log::init_log());

  let builder = builder.setup(|app| {
    // 获取应用数据目录
    // app_data_dir() 返回Roaming目录，不适合存放数据库文件
    // app_local_data_dir() 返回Local目录，适合存放数据库文件
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| {
      tauri_plugin_log::log::error!("failed to get app data dir, error: {e}");
      e
    })?;

    // 初始化数据库状态
    app.manage(db::state::DbState::new(app_data_dir).with_migrators(vec![db_demo::migrate]));

    // 注册托盘菜单；进程级数据库会话在 setup 里挂上（此时才有 AppHandle）。
    #[cfg(desktop)]
    if let Err(e) = menu::tray::register_tray_menu(app) {
      tauri_plugin_log::log::error!("tray menu registration failed: {e}");
    }

    Ok(())
  });

  // 注册命令
  let builder = builder.invoke_handler(tauri::generate_handler![
    db::commands::db_status,
    db::commands::db_create,
    db::commands::db_unlock,
    db::commands::db_lock,
    db_demo::commands::demo_list,
    db_demo::commands::demo_get,
    db_demo::commands::demo_create,
    db_demo::commands::demo_update,
    db_demo::commands::demo_delete,
  ]);

  // 桌面：关闭窗口时隐藏到托盘；移动端不注册，交给系统默认关闭行为
  #[cfg(desktop)]
  let builder = builder.on_window_event(|window, event| {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
      api.prevent_close();
      let _ = window.hide();
    }
  });

  // 构建应用
  let app = builder.build(tauri::generate_context!()).unwrap();
  app.run(|app_handle, event| match event {
    tauri::RunEvent::ExitRequested { .. } => {
      app_handle.state::<db::state::DbState>().lock();
    }
    _ => {}
  });
}
