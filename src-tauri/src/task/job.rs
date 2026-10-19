//! 侧车任务的执行期句柄：查取消，并把进度写回队列。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter, Manager};

use super::constants::EVENT_TASK;
use super::queue::TaskQueue;

/// 任务执行期句柄
///
/// 用于在任务执行期间报告进度和消息。
pub struct JobCtx {
  app: AppHandle,
  /// 任务 ID
  id: String,
  /// 取消标志
  cancel: Arc<AtomicBool>,
}

impl JobCtx {
  /// 组装任务执行期句柄
  ///
  /// # 参数
  /// - `app`: Tauri 应用句柄
  /// - `id`: 任务 ID
  /// - `cancel`: 取消标志
  ///
  /// # 返回值
  /// 新的 `JobCtx`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let job = JobCtx::new(app, id, cancel);
  /// ```
  pub fn new(app: AppHandle, id: impl Into<String>, cancel: Arc<AtomicBool>) -> Self {
    Self { app, id: id.into(), cancel }
  }

  /// 读取取消标志
  ///
  /// # 返回值
  /// 标志为真时返回 `true`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// if job.is_canceled().await {
  ///   return;
  /// }
  /// ```
  pub async fn is_canceled(&self) -> bool {
    self.cancel.load(Ordering::Relaxed)
  }

  /// 记下进度和消息，并在任务仍在处理时发出 `task-update`
  ///
  /// 队列里找不到这条，或状态不是处理中时，不发事件。
  ///
  /// # 参数
  /// - `progress`: 进度数值
  /// - `message`: 进度说明
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// job.report(40.0, "转码中...").await;
  /// ```
  pub async fn report(&self, progress: f64, message: &str) {
    let queue = self.app.state::<TaskQueue>();
    if let Ok(Some(task)) = queue.note(&self.id, progress, message) {
      let _ = self.app.emit(EVENT_TASK, &task);
    }
  }

  /// 把当前值和总量换成 0 到 100 的进度后再报告
  ///
  /// `total` 小于等于 0 时进度记为 0，否则按 `current / total * 100` 并夹在 0 到 100。
  ///
  /// # 参数
  /// - `current`: 当前进度
  /// - `total`: 总量
  /// - `message`: 进度说明
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// job.report_ratio(current_time, duration, "转码中...").await;
  /// ```
  pub async fn report_ratio(&self, current: f64, total: f64, message: &str) {
    let progress = if total <= 0.0 { 0.0 } else { ((current / total) * 100.0).clamp(0.0, 100.0) };
    self.report(progress, message).await;
  }
}
