use serde::Serialize;

/// 设置页展示的版本
///
/// 读不到的项用「未知」「不可用」「未包含」或「超时」，命令本身不失败。
///
/// # 字段
/// - `app`: 应用版本，来自包信息
/// - `webview`: WebView 版本
/// - `ffmpeg`: FFmpeg 版本
/// - `cjxl`: cjxl 版本
/// - `avifenc`: avifenc 版本
///
/// # 示例
/// 见 [`app_versions`](super::commands::app_versions)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppVersions {
  /// 应用版本，来自包信息
  pub app: String,
  /// WebView 版本
  pub webview: String,
  /// FFmpeg 版本
  pub ffmpeg: String,
  /// cjxl 版本
  pub cjxl: String,
  /// avifenc 版本
  pub avifenc: String,
}
