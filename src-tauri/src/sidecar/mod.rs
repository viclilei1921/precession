//! 桌面侧车进程：错误类型，以及当前子进程句柄。

use std::sync::Mutex;

use tauri::Manager;
use tauri_plugin_shell::process::CommandChild;

use crate::task::JobCtx;

#[derive(Debug, thiserror::Error)]
pub enum SidecarError {
  #[error("已取消")]
  Canceled,
  #[error("{0}")]
  Invalid(String),
  #[error("{0}")]
  Io(String),
  #[error("{name} 启动失败: {detail}")]
  Spawn { name: String, detail: String },
  #[error("{name} {detail}")]
  Exit { name: String, detail: String },
}

impl SidecarError {
  pub fn canceled() -> Self {
    Self::Canceled
  }

  pub fn invalid(message: impl Into<String>) -> Self {
    Self::Invalid(message.into())
  }

  pub fn io(message: impl Into<String>) -> Self {
    Self::Io(message.into())
  }

  pub fn sidecar_spawn(name: impl Into<String>, err: impl std::fmt::Display) -> Self {
    Self::Spawn { name: name.into(), detail: err.to_string() }
  }

  pub fn sidecar_exit(name: impl Into<String>, detail: impl Into<String>) -> Self {
    Self::Exit { name: name.into(), detail: detail.into() }
  }
}

impl From<std::io::Error> for SidecarError {
  fn from(err: std::io::Error) -> Self {
    Self::Io(err.to_string())
  }
}

pub type SidecarResult<T> = Result<T, SidecarError>;

/// 当前正在跑的侧车子进程。取消任务时把它杀掉，避免输出通道一直堵着。
pub struct ProcessSlot {
  child: Mutex<Option<CommandChild>>,
}

impl Default for ProcessSlot {
  fn default() -> Self {
    Self { child: Mutex::new(None) }
  }
}

impl ProcessSlot {
  fn lock(&self) -> std::sync::MutexGuard<'_, Option<CommandChild>> {
    self.child.lock().unwrap_or_else(|err| err.into_inner())
  }

  pub fn store(&self, child: CommandChild) {
    let mut slot = self.lock();
    if let Some(previous) = slot.replace(child) {
      drop(slot);
      kill_child(previous);
    }
  }

  pub fn clear(&self) {
    if let Some(child) = self.lock().take() {
      kill_child(child);
    }
  }

  pub fn kill(&self) {
    if let Some(child) = self.lock().take() {
      kill_child(child);
    }
  }
}

fn kill_child(child: CommandChild) {
  // macOS 上在异步运行时里直接 kill 可能崩溃，放到阻塞线程。
  tauri::async_runtime::spawn_blocking(move || {
    let _ = child.kill();
  });
}

pub async fn secure_spawn_step(app: &tauri::AppHandle, job: &JobCtx, child: CommandChild) -> SidecarResult<()> {
  if job.is_canceled().await {
    kill_child(child);
    return Err(SidecarError::canceled());
  }
  app.state::<ProcessSlot>().store(child);
  Ok(())
}

pub async fn clear_current_child(app: &tauri::AppHandle) {
  app.state::<ProcessSlot>().clear();
}
