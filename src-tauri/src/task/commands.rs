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
/// 桌面端且取消结果要求杀掉子进程时，调用进程槽的 `kill`。随后发出 `task-update` 并发布队列快照。
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
  #[cfg(desktop)]
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
/// 桌面端交给 `reveal_in_folder`。非桌面直接返回 `TaskError::DesktopOnly`。
///
/// # 参数
/// - `path`: 要打开的文件或目录
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 非桌面返回 `TaskError::DesktopOnly`。桌面端失败时返回 `reveal_in_folder` 的 `TaskError`
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
  #[cfg(desktop)]
  {
    return reveal_in_folder(&path);
  }
  #[cfg(not(desktop))]
  {
    let _ = path;
    Err(TaskError::DesktopOnly)
  }
}

/// 用 macOS 的 `open` 打开文件或其所在目录
///
/// 路径是文件时加 `-R` 选中它。否则父目录存在就打开父目录。
///
/// # 参数
/// - `path`: 要打开的路径
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 既不是文件、父目录也不存在时返回 `TaskError::Missing`。`open` 启动失败时返回 `TaskError::Failed`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// reveal_in_folder(path)?;
/// ```
#[cfg(target_os = "macos")]
fn reveal_in_folder(path: &str) -> Result<(), TaskError> {
  let target = std::path::Path::new(path);
  let mut command = std::process::Command::new("open");
  if target.is_file() {
    command.args(["-R", path]);
  } else if let Some(parent) = target.parent().filter(|parent| parent.is_dir()) {
    command.arg(parent);
  } else {
    return Err(TaskError::Missing);
  }
  command.spawn().map_err(|err| TaskError::Failed(err.to_string()))?;
  Ok(())
}

/// 用资源管理器打开文件或其所在目录
///
/// 路径是文件时带 `/select,` 选中它。否则父目录存在就打开父目录。
///
/// # 参数
/// - `path`: 要打开的路径
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 既不是文件、父目录也不存在时返回 `TaskError::Missing`。`explorer` 启动失败时返回 `TaskError::Failed`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// reveal_in_folder(path)?;
/// ```
#[cfg(target_os = "windows")]
fn reveal_in_folder(path: &str) -> Result<(), TaskError> {
  use std::os::windows::process::CommandExt;

  let target = std::path::Path::new(path);
  let mut command = std::process::Command::new("explorer");
  if target.is_file() {
    command.raw_arg(format!("/select,{path}"));
  } else if let Some(parent) = target.parent().filter(|parent| parent.is_dir()) {
    command.arg(parent);
  } else {
    return Err(TaskError::Missing);
  }
  command.spawn().map_err(|err| TaskError::Failed(err.to_string()))?;
  Ok(())
}

/// 用 `xdg-open` 打开目录
///
/// 路径本身是目录就打开它，否则打开父目录；没有父目录时仍把原路径交给 `xdg-open`。
///
/// # 参数
/// - `path`: 要打开的路径
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// `xdg-open` 启动失败时返回 `TaskError::Failed`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// reveal_in_folder(path)?;
/// ```
#[cfg(all(desktop, not(any(target_os = "macos", target_os = "windows"))))]
fn reveal_in_folder(path: &str) -> Result<(), TaskError> {
  let target = std::path::Path::new(path);
  let folder = if target.is_dir() { target } else { target.parent().unwrap_or(target) };
  std::process::Command::new("xdg-open").arg(folder).spawn().map_err(|err| TaskError::Failed(err.to_string()))?;
  Ok(())
}
