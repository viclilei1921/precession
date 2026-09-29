//! 侧车任务的执行期句柄：查取消，并把进度写回队列。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter, Manager};

use super::constants::EVENT_TASK;
use super::queue::TaskQueue;

pub struct JobCtx {
  app: AppHandle,
  id: String,
  cancel: Arc<AtomicBool>,
}

impl JobCtx {
  pub fn new(app: AppHandle, id: impl Into<String>, cancel: Arc<AtomicBool>) -> Self {
    Self { app, id: id.into(), cancel }
  }

  pub async fn is_canceled(&self) -> bool {
    self.cancel.load(Ordering::Relaxed)
  }

  pub async fn report(&self, progress: f64, message: &str) {
    let queue = self.app.state::<TaskQueue>();
    if let Ok(Some(task)) = queue.note(&self.id, progress, message) {
      let _ = self.app.emit(EVENT_TASK, &task);
    }
  }

  pub async fn report_ratio(&self, current: f64, total: f64, message: &str) {
    let progress = if total <= 0.0 { 0.0 } else { ((current / total) * 100.0).clamp(0.0, 100.0) };
    self.report(progress, message).await;
  }
}
