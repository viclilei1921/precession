use tauri::AppHandle;

use super::dto::AppVersions;
use super::error::AboutError;
use super::service;

/// 返回应用、WebView 和三个侧车的版本
///
/// 侧车在桌面并行启动，每个最多等 5 秒。不写入转码用的进程槽。手机端侧车返回「未包含」。
///
/// # 参数
/// - `app`: Tauri 应用句柄
///
/// # 返回值
/// 成功时返回 [`AppVersions`]
///
/// # 错误
/// 函数体始终返回 `Ok`，不会产生 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let versions = app_versions(app).await?;
/// ```
#[tauri::command]
pub async fn app_versions(app: AppHandle) -> Result<AppVersions, AboutError> {
  service::versions(&app).await
}
