use std::thread;

use tauri::{AppHandle, Emitter, Manager};

use super::constants::{EVENT_QUEUE, EVENT_TASK};
use super::error::TaskError;
use super::queue::TaskQueue;
use super::service;
use crate::db::state::DbState;

/// 启动名为 `task-worker` 的后台线程并循环取任务
///
/// `pump` 失败只记日志，线程继续循环。线程创建失败也只记日志。
///
/// # 参数
/// - `app`: Tauri 应用句柄
///
/// # 返回值
/// 无返回值
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// spawn_worker(app);
/// ```
pub(crate) fn spawn_worker(app: AppHandle) {
  let built = thread::Builder::new().name("task-worker".to_string()).spawn(move || {
    loop {
      if let Err(err) = pump(&app) {
        tauri_plugin_log::log::error!("task worker: {err}");
      }
    }
  });
  if let Err(err) = built {
    tauri_plugin_log::log::error!("task worker thread: {err}");
  }
}

/// 取一张任务票，执行后收尾并发布队列
///
/// 取到票后先把进度记为 0、消息写成「准备开始...」，执行中的进度再写回并发出 `task-update`。
///
/// # 参数
/// - `app`: Tauri 应用句柄
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 等票或收尾失败时返回 `TaskError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// pump(&app)?;
/// ```
fn pump(app: &AppHandle) -> Result<(), TaskError> {
  let queue = app.state::<TaskQueue>();
  let ticket = queue.wait_ticket()?;
  if let Ok(Some(task)) = queue.note(&ticket.id, 0.0, "准备开始...") {
    let _ = app.emit(EVENT_TASK, &task);
  }
  let db = app.state::<DbState>().inner().clone();
  let id = ticket.id.clone();
  let app_emit = app.clone();
  let result = service::run_job(app, &db, &ticket, &mut |progress, message| {
    if let Ok(Some(task)) = queue.note(&id, progress, message) {
      let _ = app_emit.emit(EVENT_TASK, &task);
    }
  });
  let task = queue.finish(&id, &result)?;
  let _ = app.emit(EVENT_TASK, &task);
  publish_queue(app);
  Ok(())
}

/// 读取队列并在成功时发出 `queue-updated`
///
/// 列出失败时不发事件。
///
/// # 参数
/// - `app`: Tauri 应用句柄
///
/// # 返回值
/// 无返回值
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// publish_queue(&app);
/// ```
pub(super) fn publish_queue(app: &AppHandle) {
  let queue = app.state::<TaskQueue>();
  if let Ok(list) = queue.list() {
    let _ = app.emit(EVENT_QUEUE, &list);
  }
}
