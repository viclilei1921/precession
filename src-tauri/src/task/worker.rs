use std::thread;

use tauri::{AppHandle, Emitter, Manager};

use super::constants::{EVENT_QUEUE, EVENT_TASK};
use super::error::TaskError;
use super::queue::TaskQueue;
use super::service;
use crate::db::state::DbState;

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

fn pump(app: &AppHandle) -> Result<(), TaskError> {
  let queue = app.state::<TaskQueue>();
  let ticket = queue.wait_ticket()?;
  if let Ok(Some(task)) = queue.note(&ticket.id, 0.0, "准备开始...") {
    let _ = app.emit(EVENT_TASK, &task);
  }
  let db = app.state::<DbState>().inner().clone();
  let id = ticket.id.clone();
  let app_emit = app.clone();
  let result = service::run(&db, &ticket, &mut |progress, message| {
    if let Ok(Some(task)) = queue.note(&id, progress, message) {
      let _ = app_emit.emit(EVENT_TASK, &task);
    }
  });
  let task = queue.finish(&id, &result)?;
  let _ = app.emit(EVENT_TASK, &task);
  publish_queue(app);
  Ok(())
}

pub(super) fn publish_queue(app: &AppHandle) {
  let queue = app.state::<TaskQueue>();
  if let Ok(list) = queue.list() {
    let _ = app.emit(EVENT_QUEUE, &list);
  }
}
