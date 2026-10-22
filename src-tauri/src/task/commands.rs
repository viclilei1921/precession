use tauri::{AppHandle, Emitter, Manager};

use super::constants::EVENT_TASK;
use super::dto::{Task, TaskInput};
use super::error::TaskError;
use super::queue::TaskQueue;
use super::service;
use super::worker;
use crate::db::state::DbState;

/// 把任务放进队列并发布队列快照
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `input`: 待入队的任务参数
///
/// # 返回值
/// 成功时返回已入队的任务
///
/// # 错误
/// 准备或入队失败时返回 `TaskError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let task = task_enqueue(app, input).await?;
/// ```
#[tauri::command]
pub async fn task_enqueue(app: AppHandle, input: TaskInput) -> Result<Task, TaskError> {
  let db = app.state::<DbState>();
  let queue = app.state::<TaskQueue>();
  let task = service::enqueue(&db, &queue, input)?;
  worker::publish_queue(&app);
  Ok(task)
}

/// 列出队列里的全部任务
///
/// # 参数
/// - `app`: Tauri 应用句柄
///
/// # 返回值
/// 成功时返回任务列表
///
/// # 错误
/// 锁定队列失败时返回 `TaskError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let tasks = task_list(app).await?;
/// ```
#[tauri::command]
pub async fn task_list(app: AppHandle) -> Result<Vec<Task>, TaskError> {
  app.state::<TaskQueue>().list()
}

/// 取消一条任务，必要时杀掉当前侧车子进程
///
/// 取消结果要求杀掉子进程时调用进程槽的 `kill`。手机端这个调用什么都不做。随后发出 `task-update` 并发布队列快照。
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `id`: 任务 ID
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 找不到任务时返回 `TaskError::NotFound`，锁定队列失败时返回 `TaskError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// task_cancel(app, id).await?;
/// ```
#[tauri::command]
pub async fn task_cancel(app: AppHandle, id: String) -> Result<(), TaskError> {
  let outcome = app.state::<TaskQueue>().cancel(&id)?;
  if outcome.kill_child {
    app.state::<crate::sidecar::ProcessSlot>().kill();
  }
  let _ = app.emit(EVENT_TASK, &outcome.task);
  worker::publish_queue(&app);
  Ok(())
}

/// 从队列拿掉一条任务并发布队列快照
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `id`: 任务 ID
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 找不到任务时返回 `TaskError::NotFound`，正在处理时返回 `TaskError::Running`，锁定失败时返回 `TaskError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// task_remove(app, id).await?;
/// ```
#[tauri::command]
pub async fn task_remove(app: AppHandle, id: String) -> Result<(), TaskError> {
  app.state::<TaskQueue>().remove(&id)?;
  worker::publish_queue(&app);
  Ok(())
}

/// 在系统文件管理器中打开路径
///
/// 桌面端交给当前平台。非桌面直接返回 `TaskError::DesktopOnly`。
///
/// # 参数
/// - `path`: 要打开的文件或目录
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 非桌面返回 `TaskError::DesktopOnly`。桌面端失败时返回平台打开文件的 `TaskError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// task_reveal(path).await?;
/// ```
#[tauri::command]
pub async fn task_reveal(path: String) -> Result<(), TaskError> {
  super::platform::reveal(&path)
}
