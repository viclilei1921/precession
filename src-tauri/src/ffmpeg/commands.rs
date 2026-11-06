use tauri::AppHandle;

use super::models::{VideoEncode, VideoInfo};
use super::preview::QualityPreview;
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
  super::platform::probe(&app, &path).await
}

/// 用当前画面参数预估中间一小段的画质
///
/// 非桌面返回 `DesktopOnly`。桌面取中间约 2.5 秒转码，再算 VMAF 和 SSIM。
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `path`: 视频路径
/// - `encode`: 编码配置。空则用默认画面参数
///
/// # 返回值
/// 成功时返回取样位置、分数、分档和按这段推算的体积
///
/// # 错误
/// 非桌面返回 `TaskError::DesktopOnly`。侧车取消变成 `Canceled`，其他侧车错误变成 `Failed`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let preview = ffmpeg_preview_quality(app, path, None).await?;
/// ```
#[tauri::command]
pub async fn ffmpeg_preview_quality(
  app: AppHandle,
  path: String,
  encode: Option<VideoEncode>,
) -> Result<QualityPreview, TaskError> {
  super::preview::preview_quality(&app, path, encode).await
}

/// 停掉正在进行的画质预估
///
/// 没有正在跑的预估时也成功。
///
/// # 参数
/// - `app`: Tauri 应用句柄
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 不返回错误
///
/// # Panics
/// 桌面端没有登记预估进程槽时，`state` 会 panic
///
/// # 示例
/// ```ignore
/// ffmpeg_preview_cancel(app).await?;
/// ```
#[tauri::command]
pub async fn ffmpeg_preview_cancel(app: AppHandle) -> Result<(), TaskError> {
  super::preview::preview_cancel(&app);
  Ok(())
}
