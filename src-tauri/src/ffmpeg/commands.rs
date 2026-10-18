use tauri::AppHandle;

use super::models::VideoInfo;
use crate::task::error::TaskError;

/// 读取视频时长和分辨率。执行只在桌面。
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
