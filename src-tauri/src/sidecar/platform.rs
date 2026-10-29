//! 侧车进程在各平台上怎么跟踪子进程。非桌面这些入口什么都不做。

/// 桌面：记住当前子进程，取消时杀掉。
#[cfg(desktop)]
mod imp {
  use tauri::Manager;
  use tauri_plugin_shell::process::CommandChild;

  use crate::task::JobCtx;

  use super::super::error::{SidecarError, SidecarResult};
  use super::super::slot::ProcessSlot;

  /// 锁住进程槽，中毒时取回里面的数据
  ///
  /// # 参数
  /// - `slot`: 进程槽
  ///
  /// # 返回值
  /// 子进程槽的互斥锁
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let guard = lock(slot);
  /// ```
  fn lock(slot: &ProcessSlot) -> std::sync::MutexGuard<'_, Option<CommandChild>> {
    slot.child.lock().unwrap_or_else(|err| err.into_inner())
  }

  /// 在阻塞线程里杀掉这个子进程
  ///
  /// macOS 上在异步运行时里直接杀可能崩溃，所以放到 `spawn_blocking`。
  ///
  /// # 参数
  /// - `child`: 子进程
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// kill_child(child);
  /// ```
  fn kill_child(child: CommandChild) {
    // macOS 上在异步运行时里直接 kill 可能崩溃，放到阻塞线程。
    tauri::async_runtime::spawn_blocking(move || {
      let _ = child.kill();
    });
  }

  /// 记下子进程，槽里已有的先杀掉
  ///
  /// # 参数
  /// - `slot`: 进程槽
  /// - `child`: 新的子进程
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// store(slot, child);
  /// ```
  pub fn store(slot: &ProcessSlot, child: CommandChild) {
    let mut guard = lock(slot);
    if let Some(previous) = guard.replace(child) {
      drop(guard);
      kill_child(previous);
    }
  }

  /// 取出并杀掉槽里的子进程
  ///
  /// # 参数
  /// - `slot`: 进程槽
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// clear(slot);
  /// ```
  pub fn clear(slot: &ProcessSlot) {
    // 如果当前有子进程，杀死它
    if let Some(child) = lock(slot).take() {
      kill_child(child);
    }
  }

  /// 取出并杀掉槽里的子进程
  ///
  /// # 参数
  /// - `slot`: 进程槽
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// kill(slot);
  /// ```
  pub fn kill(slot: &ProcessSlot) {
    if let Some(child) = lock(slot).take() {
      kill_child(child);
    }
  }

  /// 任务已取消时杀掉子进程，否则放进进程槽
  ///
  /// # 参数
  /// - `app`: Tauri 应用句柄
  /// - `job`: 任务执行期句柄
  /// - `child`: 已启动的子进程
  ///
  /// # 返回值
  /// 成功时无额外值
  ///
  /// # 错误
  /// 任务已取消时返回 `SidecarError::Canceled`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// secure_spawn_step(app, job, child).await?;
  /// ```
  pub async fn secure_spawn_step(app: &tauri::AppHandle, job: &JobCtx, child: CommandChild) -> SidecarResult<()> {
    // 如果任务被取消，杀死子进程并返回错误
    if job.is_canceled().await {
      kill_child(child);
      return Err(SidecarError::canceled());
    }

    // 存储子进程
    app.state::<ProcessSlot>().store(child);
    Ok(())
  }

  /// 清掉应用状态里的当前子进程
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
  /// clear_current_child(app).await;
  /// ```
  pub async fn clear_current_child(app: &tauri::AppHandle) {
    app.state::<ProcessSlot>().clear();
  }
}

/// 其他平台：这些入口什么都不做。
#[cfg(not(desktop))]
mod imp {
  use tauri_plugin_shell::process::CommandChild;

  use crate::task::JobCtx;

  use super::super::error::SidecarResult;
  use super::super::slot::ProcessSlot;

  /// 在非桌面忽略子进程
  ///
  /// # 参数
  /// - `slot`: 进程槽
  /// - `child`: 子进程
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// store(slot, child);
  /// ```
  pub fn store(slot: &ProcessSlot, child: CommandChild) {
    let _ = (slot, child);
  }

  /// 在非桌面什么都不做
  ///
  /// # 参数
  /// - `slot`: 进程槽
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// clear(slot);
  /// ```
  pub fn clear(slot: &ProcessSlot) {
    let _ = slot;
  }

  /// 在非桌面什么都不做
  ///
  /// # 参数
  /// - `slot`: 进程槽
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// kill(slot);
  /// ```
  pub fn kill(slot: &ProcessSlot) {
    let _ = slot;
  }

  /// 在非桌面忽略子进程并直接成功
  ///
  /// # 参数
  /// - `app`: Tauri 应用句柄
  /// - `job`: 任务执行期句柄
  /// - `child`: 子进程
  ///
  /// # 返回值
  /// 成功时无额外值
  ///
  /// # 错误
  /// 不返回错误
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// secure_spawn_step(app, job, child).await?;
  /// ```
  pub async fn secure_spawn_step(app: &tauri::AppHandle, job: &JobCtx, child: CommandChild) -> SidecarResult<()> {
    let _ = (app, job, child);
    Ok(())
  }

  /// 在非桌面什么都不做
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
  /// clear_current_child(app).await;
  /// ```
  pub async fn clear_current_child(app: &tauri::AppHandle) {
    let _ = app;
  }
}

pub(super) use imp::{clear, clear_current_child, kill, secure_spawn_step, store};
