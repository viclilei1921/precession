use tauri::AppHandle;

use super::models::VideoInfo;
use crate::task::error::TaskError;

/// 读取视频信息
///
/// 非桌面返回 `DesktopOnly`。桌面调用 `video_info`。
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `path`: 视频路径
///
/// # 返回值
/// 成功时返回视频信息
///
/// # 错误
/// 非桌面返回 `TaskError::DesktopOnly`。侧车取消变成 `Canceled`，其他侧车错误变成 `Failed`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let info = ffmpeg_probe(app, path).await?;
/// ```
#[tauri::command]
pub async fn ffmpeg_probe(app: AppHandle, path: String) -> Result<VideoInfo, TaskError> {
  #[cfg(not(desktop))]
  {
    let _ = (app, path);
    Err(TaskError::DesktopOnly)
  }
  #[cfg(desktop)]
  {
    super::video_info(&app, &path).await.map_err(TaskError::from)
  }
}
