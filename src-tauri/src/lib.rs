mod catalog;
mod constants;
mod crypto;
mod db;
mod db_demo;
mod library;
#[cfg(desktop)]
mod menu;
mod plan;
mod plugin;
mod record;
mod utils;

use tauri::Manager;

#[cfg(target_os = "android")]
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let builder = tauri::Builder::default();

  // 初始化日志
  let builder = builder.plugin(plugin::log::init_log());

  #[cfg(target_os = "android")]
  let builder = builder.plugin(precession_device_slot::init());

  let builder = builder.setup(|app| {
    // 获取应用数据目录
    // app_data_dir() 返回Roaming目录，不适合存放数据库文件
    // app_local_data_dir() 返回Local目录，适合存放数据库文件
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| {
      tauri_plugin_log::log::error!("failed to get app data dir, error: {e}");
      e
    })?;

    // 初始化数据库状态
    let db = db::state::DbState::new(app_data_dir).with_migrators(vec![
      db_demo::migrate,
      catalog::migrate,
      record::migrate,
      plan::migrate,
      library::migrate,
    ]);
    #[cfg(target_os = "android")]
    let db = {
      let api = app.state::<precession_device_slot::DeviceApi<tauri::Wry>>().inner().clone();
      db.with_device(Arc::new(crate::crypto::device::AndroidDevice::new(api)))
    };
    app.manage(db);

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
    db::commands::db_enable_device_unlock,
    db::commands::db_unlock_device,
    db::commands::db_disable_device_unlock,
    db_demo::commands::demo_list,
    db_demo::commands::demo_get,
    db_demo::commands::demo_create,
    db_demo::commands::demo_update,
    db_demo::commands::demo_delete,
    catalog::commands::member_list,
    catalog::commands::member_get,
    catalog::commands::member_create,
    catalog::commands::member_update,
    catalog::commands::member_delete,
    catalog::commands::tag_list,
    catalog::commands::tag_create,
    catalog::commands::tag_update,
    catalog::commands::tag_delete,
    catalog::commands::place_list,
    catalog::commands::place_create,
    catalog::commands::place_update,
    catalog::commands::place_delete,
    record::commands::record_list,
    record::commands::record_get,
    record::commands::record_create,
    record::commands::record_update,
    record::commands::record_delete,
    record::commands::media_list,
    record::commands::media_create,
    record::commands::media_delete,
    record::commands::record_link_list,
    record::commands::record_link_create,
    record::commands::record_link_delete,
    plan::commands::plan_list,
    plan::commands::plan_get,
    plan::commands::plan_create,
    plan::commands::plan_update,
    plan::commands::plan_complete,
    plan::commands::plan_delete,
    library::commands::book_list,
    library::commands::book_get,
    library::commands::book_create,
    library::commands::book_update,
    library::commands::book_delete,
    library::commands::quote_list,
    library::commands::quote_get,
    library::commands::quote_create,
    library::commands::quote_update,
    library::commands::quote_delete,
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
      app_handle.state::<db::state::DbState>().release_session();
    }
    _ => {}
  });
}
