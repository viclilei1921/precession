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
