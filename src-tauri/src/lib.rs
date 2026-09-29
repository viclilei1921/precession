mod book;
mod calendar;
mod crypto;
mod db;
mod ffmpeg;
mod growth;
mod image;
mod journal;
mod library;
mod media;
mod member;
mod owner;
mod place;
mod plan;
mod plugin;
#[cfg(desktop)]
mod sidecar;
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
  let builder = tauri::Builder::default().plugin(tauri_plugin_os::init());

  // 初始化日志
  let builder = builder.plugin(plugin::log::init_log());

  // 桌面：初始化 shell 插件(为了侧车功能)
  #[cfg(desktop)]
  let builder = builder.plugin(tauri_plugin_shell::init());

  // android端：初始化设备插槽插件(为了设备锁功能)
  #[cfg(target_os = "android")]
  let builder = builder.plugin(precession_device_slot::init());

  let builder = builder.setup(|app| {
    // 获取应用数据目录
    // app_data_dir() 返回Roaming目录，不适合存放数据库文件
    // app_local_data_dir() 返回Local目录，适合存放数据库文件
    tauri_plugin_log::log::info!("app path: {:?}", app.path().app_local_data_dir());
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

    // android端：初始化设备插槽(为了设备锁功能)
    #[cfg(target_os = "android")]
    let db = {
      let api = app.state::<precession_device_slot::DeviceApi<tauri::Wry>>().inner().clone();
      db.with_device(Arc::new(crate::crypto::device::AndroidDevice::new(api)))
    };

    // 管理数据库状态
    app.manage(db);

    // 管理任务队列
    app.manage(task::TaskQueue::default());

    // 桌面：管理侧车进程
    #[cfg(desktop)]
    app.manage(sidecar::ProcessSlot::default());

    // 启动任务队列工作线程
    task::spawn_worker(app.handle().clone());

    // 注册托盘菜单；进程级数据库会话在 setup 里挂上（此时才有 AppHandle）。
    #[cfg(desktop)]
    if let Err(e) = tray::register_tray_menu(app) {
      tauri_plugin_log::log::error!("tray menu registration failed: {e}");
    }

    // macOS：关掉 WKWebView 根滚动视图的弹性回弹，避免内容不够长时仍能拖出白边。
    #[cfg(target_os = "macos")]
    match app.get_webview_window("main") {
      Some(window) => {
        if let Err(error) = window.with_webview(|webview| unsafe {
          let view: &objc2_web_kit::WKWebView = &*webview.inner().cast();
          disable_elastic_overscroll(view);
          let ns_window: &objc2_app_kit::NSWindow = &*webview.ns_window().cast();
          align_traffic_lights(ns_window);
        }) {
          tauri_plugin_log::log::error!("disable webview elastic overscroll failed: {error}");
        }
      }
      None => {
        tauri_plugin_log::log::error!("main window missing, elastic overscroll left enabled");
      }
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
    plan::commands::plan_group_list,
    plan::commands::plan_group_create,
    plan::commands::plan_group_update,
    plan::commands::plan_group_delete,
    plan::commands::plan_comment_create,
    plan::commands::plan_comment_update,
    plan::commands::plan_comment_delete,
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

    // 系统会在缩放后把红绿灯放回默认位置，重新对准标题栏中线。
    // 全屏时按钮由系统收进屏幕顶栏，这里不要改。
    #[cfg(target_os = "macos")]
    if let tauri::WindowEvent::Resized(_) = event {
      if window.is_fullscreen().ok() == Some(true) {
        return;
      }
      align_main_traffic_lights(window);
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

/// 标题栏高度，和前端 `1.75rem` 一致。红绿灯中线和标题中线对齐。
#[cfg(target_os = "macos")]
const TITLEBAR_HEIGHT: f64 = 28.0;

#[cfg(target_os = "macos")]
fn align_main_traffic_lights(window: &tauri::Window) {
  let Some(webview) = window.get_webview_window(window.label()) else {
    return;
  };
  let _ = webview.with_webview(|webview| unsafe {
    let ns_window: &objc2_app_kit::NSWindow = &*webview.ns_window().cast();
    align_traffic_lights(ns_window);
  });
}

/// 把关闭、最小化、缩放按钮的中线放到标题栏中线。只改纵向位置。
#[cfg(target_os = "macos")]
fn align_traffic_lights(window: &objc2_app_kit::NSWindow) {
  use objc2_app_kit::NSWindowButton;
  use objc2_foundation::NSPoint;

  let Some(content) = window.contentView() else {
    return;
  };
  let Some(close) = window.standardWindowButton(NSWindowButton::CloseButton) else {
    return;
  };
  let Some(button_super) = (unsafe { close.superview() }) else {
    return;
  };

  let button_height = close.frame().size.height;
  let center_y = content.bounds().size.height - TITLEBAR_HEIGHT / 2.0;
  let center = content.convertPoint_toView(NSPoint::new(0.0, center_y), Some(&*button_super));
  let origin_y = center.y - button_height / 2.0;

  for kind in [NSWindowButton::CloseButton, NSWindowButton::MiniaturizeButton, NSWindowButton::ZoomButton] {
    let Some(button) = window.standardWindowButton(kind) else {
      continue;
    };
    let mut origin = button.frame().origin;
    origin.y = origin_y;
    button.setFrameOrigin(origin);
  }
}

/// 关掉 webview 里所有滚动视图的弹性回弹。WKWebView 的滚动视图是子视图，不是公开属性。
#[cfg(target_os = "macos")]
fn disable_elastic_overscroll(view: &objc2_app_kit::NSView) {
  use objc2_app_kit::{NSScrollElasticity, NSScrollView};

  if let Some(scroll) = view.downcast_ref::<NSScrollView>() {
    scroll.setHorizontalScrollElasticity(NSScrollElasticity::None);
    scroll.setVerticalScrollElasticity(NSScrollElasticity::None);
  }

  let subviews = view.subviews();
  for index in 0..subviews.count() {
    let child = subviews.objectAtIndex(index);
    disable_elastic_overscroll(&child);
  }
}
