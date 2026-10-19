use std::sync::Mutex;

use tauri_plugin_shell::process::CommandChild;

use crate::task::JobCtx;

use super::error::SidecarResult;
use super::platform;

/// 当前正在跑的侧车子进程。取消任务时把它杀掉，避免输出通道一直堵着。
pub struct ProcessSlot {
  pub(super) child: Mutex<Option<CommandChild>>,
}

impl Default for ProcessSlot {
  /// 创建没有子进程的进程槽
  ///
  /// # 返回值
  /// 空的 `ProcessSlot`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let slot = ProcessSlot::default();
  /// let _ = slot;
  /// ```
  fn default() -> Self {
    Self { child: Mutex::new(None) }
  }
}

impl ProcessSlot {
  /// 记下当前子进程
  ///
  /// # 参数
  /// - `child`: 侧车子进程
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// slot.store(child);
  /// ```
  pub fn store(&self, child: CommandChild) {
    platform::store(self, child);
  }

  /// 清掉当前子进程
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// slot.clear();
  /// ```
  pub fn clear(&self) {
    platform::clear(self);
  }

  /// 杀掉当前子进程
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// slot.kill();
  /// ```
  pub fn kill(&self) {
    platform::kill(self);
  }
}

/// 把已启动的子进程交给当前平台登记
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
/// 桌面端任务已取消时返回 `SidecarError::Canceled`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// secure_spawn_step(app, job, child).await?;
/// ```
pub async fn secure_spawn_step(app: &tauri::AppHandle, job: &JobCtx, child: CommandChild) -> SidecarResult<()> {
  platform::secure_spawn_step(app, job, child).await
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
  platform::clear_current_child(app).await
}
