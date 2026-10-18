use tauri::{AppHandle, Emitter, Manager};

use super::constants::EVENT_TASK;
use super::dto::{Task, TaskInput};
use super::error::TaskError;
use super::queue::TaskQueue;
use super::service;
use super::worker;
use crate::db::state::DbState;

#[tauri::command]
pub async fn task_enqueue(app: AppHandle, input: TaskInput) -> Result<Task, TaskError> {
  let db = app.state::<DbState>();
  let queue = app.state::<TaskQueue>();
  let task = service::enqueue(&db, &queue, input)?;
  worker::publish_queue(&app);
  Ok(task)
}

#[tauri::command]
pub async fn task_list(app: AppHandle) -> Result<Vec<Task>, TaskError> {
  app.state::<TaskQueue>().list()
}

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

#[tauri::command]
pub async fn task_remove(app: AppHandle, id: String) -> Result<(), TaskError> {
  app.state::<TaskQueue>().remove(&id)?;
  worker::publish_queue(&app);
  Ok(())
}

/// 在系统文件管理器中打开输出文件所在目录，并选中该文件。
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

#[cfg(all(desktop, not(any(target_os = "macos", target_os = "windows"))))]
fn reveal_in_folder(path: &str) -> Result<(), TaskError> {
  let target = std::path::Path::new(path);
  let folder = if target.is_dir() { target } else { target.parent().unwrap_or(target) };
  std::process::Command::new("xdg-open").arg(folder).spawn().map_err(|err| TaskError::Failed(err.to_string()))?;
  Ok(())
}
