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

  // 页面加载完再关一次。启动时滚动视图可能还没建好，只关那一次盖不住回弹。
  #[cfg(target_os = "macos")]
  let builder = builder.on_page_load(|webview, payload| {
    if payload.event() != tauri::webview::PageLoadEvent::Finished {
      return;
    }
    if let Err(error) = webview.with_webview(|inner| unsafe {
      let view: &objc2_web_kit::WKWebView = &*inner.inner().cast();
      disable_elastic_overscroll(view);
    }) {
      tauri_plugin_log::log::error!("disable webview elastic overscroll failed: {error}");
    }
  });

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

    // 缩放时系统会把回弹开关复位，再关掉。红绿灯在标题栏 layout 末尾对齐，这里再改会和系统位置来回抢。
    #[cfg(target_os = "macos")]
    if let tauri::WindowEvent::Resized(_) = event {
      disable_main_elastic_overscroll(window);
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

/// 标题栏高度，和前端 `.title-bar` 的 `2.25rem` 一致（根字号 16px 时为 36px）。红绿灯中线和标题中线对齐。
#[cfg(target_os = "macos")]
const TITLEBAR_HEIGHT: f64 = 36.0;

#[cfg(target_os = "macos")]
static TRAFFIC_MAIN_WINDOW: std::sync::atomic::AtomicPtr<std::ffi::c_void> =
  std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

#[cfg(target_os = "macos")]
static TRAFFIC_LAYOUT_IMP: std::sync::atomic::AtomicPtr<std::ffi::c_void> =
  std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

/// 把关闭、最小化、全屏按钮的中线放到标题栏中线。只改纵向位置。
#[cfg(target_os = "macos")]
fn align_traffic_lights(window: &objc2_app_kit::NSWindow) {
  install_traffic_light_hook(window);
  position_traffic_lights(window);
}

/// 系统在主题框的 `layout` 里把按钮放回默认高度。挂到这次布局末尾再改，绘制前就已经在中线上。
#[cfg(target_os = "macos")]
fn install_traffic_light_hook(window: &objc2_app_kit::NSWindow) {
  use std::sync::Once;

  static ONCE: Once = Once::new();
  ONCE.call_once(|| {
    let Some(host) = theme_frame_host(window) else {
      tauri_plugin_log::log::warn!("traffic light hook skipped, theme frame missing");
      return;
    };
    let Some(host_window) = host.window() else {
      return;
    };
    let ptr = objc2::rc::Retained::as_ptr(&host_window).cast::<std::ffi::c_void>().cast_mut();
    TRAFFIC_MAIN_WINDOW.store(ptr, std::sync::atomic::Ordering::Release);

    let class = host.class();
    if !hookable_theme_class(class) {
      tauri_plugin_log::log::warn!("traffic light hook skipped, class {}", class.name().to_string_lossy());
      return;
    }
    let imp = unsafe {
      std::mem::transmute::<
        unsafe extern "C-unwind" fn(*mut objc2::runtime::AnyObject, objc2::runtime::Sel),
        objc2::runtime::Imp,
      >(traffic_light_layout)
    };
    if !hook_theme_method(class, objc2::sel!(layout), imp, &TRAFFIC_LAYOUT_IMP) {
      tauri_plugin_log::log::warn!("traffic light hook failed, class {}", class.name().to_string_lossy());
    }
  });
}

#[cfg(target_os = "macos")]
fn theme_frame_host(window: &objc2_app_kit::NSWindow) -> Option<objc2::rc::Retained<objc2_app_kit::NSView>> {
  let content = window.contentView()?;
  let mut host = unsafe { content.superview() }?;
  while let Some(parent) = unsafe { host.superview() } {
    if !hookable_theme_class(parent.class()) {
      break;
    }
    host = parent;
  }
  if !hookable_theme_class(host.class()) {
    return None;
  }
  Some(host)
}

#[cfg(target_os = "macos")]
fn hookable_theme_class(class: &objc2::runtime::AnyClass) -> bool {
  let name = class.name();
  name != c"NSView" && name != c"NSResponder" && name != c"NSObject"
}

#[cfg(target_os = "macos")]
fn hook_theme_method(
  class: &objc2::runtime::AnyClass,
  sel: objc2::runtime::Sel,
  new_imp: objc2::runtime::Imp,
  slot: &std::sync::atomic::AtomicPtr<std::ffi::c_void>,
) -> bool {
  use std::sync::atomic::Ordering;

  use objc2::ffi::{
    class_addMethod, class_getInstanceMethod, method_getImplementation, method_getTypeEncoding,
    method_setImplementation,
  };

  let class_ptr = std::ptr::from_ref(class);
  let method = unsafe { class_getInstanceMethod(class_ptr, sel) };
  if method.is_null() {
    return false;
  }

  let inherited = class.superclass().is_some_and(|superclass| {
    let super_method = unsafe { class_getInstanceMethod(std::ptr::from_ref(superclass), sel) };
    super_method == method
  });

  if inherited {
    let Some(original) = (unsafe { method_getImplementation(method) }) else {
      return false;
    };
    let encoding = unsafe { method_getTypeEncoding(method) };
    if encoding.is_null() {
      return false;
    }
    slot.store(unsafe { std::mem::transmute(original) }, Ordering::Release);
    let added = unsafe { class_addMethod(class_ptr.cast_mut(), sel, new_imp, encoding) };
    if !added.as_bool() {
      slot.store(std::ptr::null_mut(), Ordering::Release);
      return false;
    }
    return true;
  }

  let Some(original) = (unsafe { method_setImplementation(method, new_imp) }) else {
    return false;
  };
  slot.store(unsafe { std::mem::transmute(original) }, Ordering::Release);
  true
}

#[cfg(target_os = "macos")]
unsafe extern "C-unwind" fn traffic_light_layout(this: *mut objc2::runtime::AnyObject, cmd: objc2::runtime::Sel) {
  if TRAFFIC_ALIGNING.with(|flag| flag.get()) {
    return;
  }
  let original = TRAFFIC_LAYOUT_IMP.load(std::sync::atomic::Ordering::Acquire);
  if !original.is_null() {
    let original: unsafe extern "C-unwind" fn(*mut objc2::runtime::AnyObject, objc2::runtime::Sel) =
      unsafe { std::mem::transmute(original) };
    unsafe { original(this, cmd) };
  }
  if this.is_null() || TRAFFIC_ALIGNING.with(|flag| flag.replace(true)) {
    return;
  }
  let _guard = ClearTrafficAligning;
  let view: &objc2_app_kit::NSView = unsafe { &*this.cast() };
  let Some(window) = view.window() else {
    return;
  };
  let current = objc2::rc::Retained::as_ptr(&window).cast::<std::ffi::c_void>();
  if current == TRAFFIC_MAIN_WINDOW.load(std::sync::atomic::Ordering::Acquire).cast_const() {
    position_traffic_lights(&window);
  }
}

#[cfg(target_os = "macos")]
thread_local! {
  static TRAFFIC_ALIGNING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(target_os = "macos")]
struct ClearTrafficAligning;

#[cfg(target_os = "macos")]
impl Drop for ClearTrafficAligning {
  fn drop(&mut self) {
    TRAFFIC_ALIGNING.with(|flag| flag.set(false));
  }
}

/// 关掉隐式动画再改原点。缩放事务里直接 `setFrameOrigin` 会把高度差做成上下滑动。
#[cfg(target_os = "macos")]
fn position_traffic_lights(window: &objc2_app_kit::NSWindow) {
  use objc2::runtime::{AnyClass, Bool};
  use objc2_app_kit::{NSWindowButton, NSWindowStyleMask};
  use objc2_foundation::NSPoint;

  if window.styleMask().contains(NSWindowStyleMask::FullScreen) {
    return;
  }
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

  let Some(transaction) = AnyClass::get(c"CATransaction") else {
    place_traffic_lights(window, origin_y);
    return;
  };
  unsafe {
    let _: () = objc2::msg_send![transaction, begin];
    let _: () = objc2::msg_send![transaction, setDisableActions: Bool::YES];
  }
  place_traffic_lights(window, origin_y);
  unsafe {
    let _: () = objc2::msg_send![transaction, commit];
  }
}

#[cfg(target_os = "macos")]
fn place_traffic_lights(window: &objc2_app_kit::NSWindow, origin_y: f64) {
  use objc2_app_kit::NSWindowButton;

  for kind in [NSWindowButton::CloseButton, NSWindowButton::MiniaturizeButton, NSWindowButton::ZoomButton] {
    let Some(button) = window.standardWindowButton(kind) else {
      continue;
    };
    let mut origin = button.frame().origin;
    if (origin.y - origin_y).abs() < 0.5 {
      continue;
    }
    origin.y = origin_y;
    button.setFrameOrigin(origin);
  }
}

#[cfg(target_os = "macos")]
fn disable_main_elastic_overscroll(window: &tauri::Window) {
  let Some(webview) = window.get_webview_window(window.label()) else {
    return;
  };
  let _ = webview.with_webview(|webview| unsafe {
    let view: &objc2_web_kit::WKWebView = &*webview.inner().cast();
    disable_elastic_overscroll(view);
  });
}

/// 关掉页面回弹，避免触控板把窗口拖出空白。
/// macOS 的 WKWebView 没有公开 scrollView，页面回弹要走 `_setRubberBandingEnabled:`。
#[cfg(target_os = "macos")]
fn disable_elastic_overscroll(view: &objc2_web_kit::WKWebView) {
  use std::sync::Once;

  use objc2::msg_send;
  use objc2::runtime::{Bool, NSObjectProtocol};
  use objc2::sel;

  let rubber_banding = sel!(_setRubberBandingEnabled:);
  if view.respondsToSelector(rubber_banding) {
    // `_WKRectEdge` 的 0 表示四边都不回弹。
    let _: () = unsafe { msg_send![view, _setRubberBandingEnabled: 0usize] };
  } else {
    static WARNED: Once = Once::new();
    WARNED.call_once(|| {
      tauri_plugin_log::log::warn!("WKWebView _setRubberBandingEnabled: unavailable");
    });
  }

  let bounce_y = sel!(_setAlwaysBounceVertical:);
  if view.respondsToSelector(bounce_y) {
    let _: () = unsafe { msg_send![view, _setAlwaysBounceVertical: Bool::NO] };
  }
  let bounce_x = sel!(_setAlwaysBounceHorizontal:);
  if view.respondsToSelector(bounce_x) {
    let _: () = unsafe { msg_send![view, _setAlwaysBounceHorizontal: Bool::NO] };
  }

  disable_scroll_view_elasticity(view);
}

/// 关掉视图树上 NSScrollView 的弹性。页面回弹不一定走这里，能找到就一起关掉。
#[cfg(target_os = "macos")]
fn disable_scroll_view_elasticity(view: &objc2_app_kit::NSView) {
  use objc2_app_kit::{NSScrollElasticity, NSScrollView};

  if let Some(scroll) = view.downcast_ref::<NSScrollView>() {
    scroll.setHorizontalScrollElasticity(NSScrollElasticity::None);
    scroll.setVerticalScrollElasticity(NSScrollElasticity::None);
  }

  let subviews = view.subviews();
  for index in 0..subviews.count() {
    let child = subviews.objectAtIndex(index);
    disable_scroll_view_elasticity(&child);
  }
}
