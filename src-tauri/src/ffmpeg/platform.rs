//! 读取视频信息。手机端直接拒绝。

/// 桌面：把探测交给侧车。
#[cfg(desktop)]
mod imp {
  use tauri::AppHandle;

  use super::super::models::VideoInfo;
  use crate::task::error::TaskError;

  /// 读取视频信息
  ///
  /// # 参数
  /// - `app`: Tauri 应用句柄
  /// - `path`: 视频路径
  ///
  /// # 返回值
  /// 成功时返回视频信息
  ///
  /// # 错误
  /// 侧车取消变成 `Canceled`，其他侧车错误变成 `Failed`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let info = probe(app, path).await?;
  /// ```
  pub async fn probe(app: &AppHandle, path: &str) -> Result<VideoInfo, TaskError> {
    super::super::video_info(app, path).await.map_err(TaskError::from)
  }
}

/// 手机端没有视频探测。
#[cfg(not(desktop))]
mod imp {
  use tauri::AppHandle;

  use super::super::models::VideoInfo;
  use crate::task::error::TaskError;

  /// 手机端不探测视频
  ///
  /// # 参数
  /// - `app`: Tauri 应用句柄，手机端不使用
  /// - `path`: 视频路径，手机端不使用
  ///
  /// # 返回值
  /// 不返回成功
  ///
  /// # 错误
  /// 始终返回 `TaskError::DesktopOnly`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let info = probe(app, path).await?;
  /// ```
  pub async fn probe(app: &AppHandle, path: &str) -> Result<VideoInfo, TaskError> {
    let _ = (app, path);
    Err(TaskError::DesktopOnly)
  }
}

pub(super) use imp::probe;
