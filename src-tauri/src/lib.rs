mod about;
mod book;
mod calendar;
mod chacha;
mod commands;
mod crypto;
mod db;
mod ffmpeg;
mod files;
mod growth;
mod image;
mod journal;
mod library;
mod member;
mod owner;
mod place;
mod plan;
mod platform;
mod plugin;
mod sidecar;
mod tag;
mod task;
mod timeline;
mod tray;
mod upload;
mod utils;

use tauri::Manager;

use platform::Platform;

/// 组装插件、数据库和任务队列并运行应用
///
/// 收到退出请求时释放数据库会话。
///
/// # 返回值
/// 不返回数据
///
/// # Panics
/// `Builder::build` 失败时 `unwrap` 会 panic
///
/// # 示例
/// ```ignore
/// run();
/// ```
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let builder = tauri::Builder::default().plugin(tauri_plugin_os::init());

  // 初始化日志
  let builder = builder.plugin(plugin::log::init_log());

  // 平台自己的插件
  let builder = platform::Current::plugins(builder);

  // 页面加载事件
  let builder = builder.on_page_load(|webview, payload| {
    platform::Current::on_page_load(webview, payload);
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
    let db = db::app_state(app_data_dir);

    // 接上本平台的系统密钥库
    let db = platform::Current::attach_device(app, db);

    // 任务工人还没启动，上次没提交的导入临时文件可以直接清掉
    upload::clear_tmp(&db);

    // 管理数据库状态
    app.manage(db);

    // 管理任务队列
    app.manage(task::TaskQueue::default());

    // 平台自己的进程和窗口状态
    platform::Current::setup(app);

    // 启动任务队列工作线程
    task::spawn_worker(app.handle().clone());

    Ok(())
  });

  // 注册命令。窗口事件交给当前平台；空实现不拦截关闭。
  let builder = builder.invoke_handler(commands::handler()).on_window_event(|window, event| {
    platform::Current::on_window_event(window, event);
  });

  // 构建应用
  let app = builder.build(tauri::generate_context!()).unwrap();
  app.run(|app_handle, event| {
    if let tauri::RunEvent::ExitRequested { .. } = event {
      app_handle.state::<db::state::DbState>().release_session();
    }
  });
}
