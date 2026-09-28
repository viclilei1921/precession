mod calendar;
mod crypto;
mod db;
mod growth;
mod journal;
mod library;
mod media;
mod member;
mod owner;
mod place;
mod plan;
mod plugin;
mod tag;
mod task;
mod timeline;
#[cfg(desktop)]
mod tray;
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
      member::migrate,
      tag::migrate,
      place::migrate,
      media::migrate,
      plan::migrate,
      journal::migrate,
      growth::migrate,
      library::migrate,
      calendar::migrate,
    ]);
    #[cfg(target_os = "android")]
    let db = {
      let api = app.state::<precession_device_slot::DeviceApi<tauri::Wry>>().inner().clone();
      db.with_device(Arc::new(crate::crypto::device::AndroidDevice::new(api)))
    };
    app.manage(db);
    app.manage(task::TaskQueue::default());
    task::spawn_worker(app.handle().clone());

    // 注册托盘菜单；进程级数据库会话在 setup 里挂上（此时才有 AppHandle）。
    #[cfg(desktop)]
    if let Err(e) = tray::register_tray_menu(app) {
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
    member::commands::member_list,
    member::commands::member_get,
    member::commands::member_create,
    member::commands::member_update,
    member::commands::member_delete,
    tag::commands::tag_list,
    tag::commands::tag_create,
    tag::commands::tag_update,
    tag::commands::tag_delete,
    place::commands::place_list,
    place::commands::place_create,
    place::commands::place_update,
    place::commands::place_delete,
    media::commands::media_list,
    media::commands::media_create,
    media::commands::media_delete,
    task::commands::task_enqueue,
    task::commands::task_list,
    task::commands::task_cancel,
    plan::commands::plan_list,
    plan::commands::plan_get,
    plan::commands::plan_create,
    plan::commands::plan_update,
    plan::commands::plan_complete,
    plan::commands::plan_delete,
    journal::commands::journal_entry_list,
    journal::commands::journal_entry_get,
    journal::commands::journal_entry_create,
    journal::commands::journal_entry_update,
    journal::commands::journal_entry_delete,
    journal::commands::journal_link_list,
    journal::commands::journal_link_create,
    journal::commands::journal_link_delete,
    journal::commands::journal_citation_list,
    journal::commands::journal_citation_create,
    journal::commands::journal_citation_delete,
    journal::commands::journal_entry_seal,
    journal::commands::journal_entry_open,
    growth::commands::growth_entry_list,
    growth::commands::growth_entry_get,
    growth::commands::growth_entry_create,
    growth::commands::growth_entry_update,
    growth::commands::growth_entry_delete,
    library::commands::book_list,
    library::commands::book_get,
    library::commands::book_create,
    library::commands::book_update,
    library::commands::book_delete,
    library::commands::book_note_list,
    library::commands::book_note_get,
    library::commands::book_note_create,
    library::commands::book_note_update,
    library::commands::book_note_delete,
    timeline::commands::timeline_list,
    calendar::commands::calendar_list,
    calendar::commands::calendar_day_upsert,
    calendar::commands::calendar_day_delete,
    calendar::commands::calendar_official_import,
    calendar::commands::calendar_almanac,
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
  app.run(|app_handle, event| {
    if let tauri::RunEvent::ExitRequested { .. } = event {
      app_handle.state::<db::state::DbState>().release_session();
    }
  });
}
