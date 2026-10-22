//! 读取应用、WebView 和侧车版本。读不到的项写成占位文本。

use tauri::AppHandle;

use super::constants::{
  AVIFENC_VERSION_HEAD, FFMPEG_VERSION_MARK, SIDECAR_AVIFENC, SIDECAR_FFMPEG, UNAVAILABLE, UNKNOWN,
};
use super::dto::AppVersions;
use super::error::AboutError;
use super::platform;

/// 三个侧车的版本文本
///
/// # 字段
/// - `ffmpeg`: FFmpeg 版本
/// - `cjxl`: cjxl 版本
/// - `avifenc`: avifenc 版本
///
/// # 示例
/// ```ignore
/// let versions = SidecarVersions {
///   ffmpeg: "7.1.1".into(),
///   cjxl: "cjxl v0.11.1".into(),
///   avifenc: "1.1.4".into(),
/// };
/// ```
#[derive(Clone, Debug)]
pub(super) struct SidecarVersions {
  /// FFmpeg 版本
  pub(super) ffmpeg: String,
  /// cjxl 版本
  pub(super) cjxl: String,
  /// avifenc 版本
  pub(super) avifenc: String,
}

/// 同一次运行只探测一次侧车
static SIDECARS: tokio::sync::OnceCell<SidecarVersions> = tokio::sync::OnceCell::const_new();

/// 读取应用、WebView 和三个侧车的版本
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
/// let versions = versions(app).await?;
/// ```
pub(super) async fn versions(app: &AppHandle) -> Result<AppVersions, AboutError> {
  let sidecars = sidecar_versions(app).await;
  Ok(AppVersions {
    app: app_label(app),
    webview: webview_label(),
    ffmpeg: sidecars.ffmpeg,
    cjxl: sidecars.cjxl,
    avifenc: sidecars.avifenc,
  })
}

/// 读取包版本
///
/// # 参数
/// - `app`: Tauri 应用句柄
///
/// # 返回值
/// `tauri.conf.json` 里的版本号
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let version = app_label(app);
/// ```
fn app_label(app: &AppHandle) -> String {
  app.package_info().version.to_string()
}

/// 读取当前 WebView 版本
///
/// 读不到时返回「未知」。
///
/// # 返回值
/// WebView 版本，失败时为「未知」
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let version = webview_label();
/// ```
fn webview_label() -> String {
  match tauri::webview_version() {
    Ok(version) => version,
    Err(err) => {
      tauri_plugin_log::log::warn!("webview version: {err}");
      UNKNOWN.to_string()
    }
  }
}

/// 取缓存的侧车版本，没有则探测一次
///
/// # 参数
/// - `app`: Tauri 应用句柄
///
/// # 返回值
/// 三个侧车的版本文本
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let versions = sidecar_versions(app).await;
/// ```
async fn sidecar_versions(app: &AppHandle) -> SidecarVersions {
  SIDECARS.get_or_init(|| platform::probe_sidecars(app.clone())).await.clone()
}

/// 从侧车输出抽出短版本
///
/// 优先 stdout 的第一行非空文本，否则用 stderr。没有文本时返回「不可用」。
///
/// # 参数
/// - `name`: 侧车名
/// - `stdout`: 标准输出
/// - `stderr`: 标准错误
///
/// # 返回值
/// 短版本字符串，解析失败时为「未知」，没有输出时为「不可用」
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let version = version_from_output(SIDECAR_FFMPEG, b"ffmpeg version 7.1.1 Copyright\n", b"");
/// assert_eq!(version, "7.1.1");
/// ```
pub(super) fn version_from_output(name: &str, stdout: &[u8], stderr: &[u8]) -> String {
  let Some(line) = first_line(stdout).or_else(|| first_line(stderr)) else {
    return UNAVAILABLE.to_string();
  };
  match name {
    SIDECAR_FFMPEG => ffmpeg_version(&line).unwrap_or_else(|| UNKNOWN.to_string()),
    SIDECAR_AVIFENC => avifenc_version(&line).unwrap_or_else(|| UNKNOWN.to_string()),
    _ => line,
  }
}

/// 取第一行非空文本
///
/// # 参数
/// - `bytes`: 进程输出
///
/// # 返回值
/// 去掉首尾空白后的第一行；没有则 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// assert_eq!(first_line(b"\nffmpeg version 7.1.1\n"), Some("ffmpeg version 7.1.1".into()));
/// ```
fn first_line(bytes: &[u8]) -> Option<String> {
  String::from_utf8_lossy(bytes).lines().map(str::trim).find(|line| !line.is_empty()).map(ToOwned::to_owned)
}

/// 抽出 `ffmpeg version` 后面的第一段
///
/// # 参数
/// - `line`: 一行 ffmpeg 输出
///
/// # 返回值
/// 版本标记；行里没有 `ffmpeg version` 时为 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// assert_eq!(ffmpeg_version("ffmpeg version 7.1.1 Copyright"), Some("7.1.1".into()));
/// ```
fn ffmpeg_version(line: &str) -> Option<String> {
  let rest = line.get(line.find(FFMPEG_VERSION_MARK)? + FFMPEG_VERSION_MARK.len()..)?;
  let token = rest.split_whitespace().next()?;
  if token.is_empty() { None } else { Some(token.to_string()) }
}

/// 抽出 `Version:` 后面的版本号
///
/// # 参数
/// - `line`: 一行 avifenc 输出
///
/// # 返回值
/// 版本号；不是 `Version:` 开头时为 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// assert_eq!(avifenc_version("Version: 1.1.4 (dav1d)"), Some("1.1.4".into()));
/// ```
fn avifenc_version(line: &str) -> Option<String> {
  let (head, tail) = line.split_once(':')?;
  if !head.trim().eq_ignore_ascii_case(AVIFENC_VERSION_HEAD) {
    return None;
  }
  let token = tail.split_whitespace().next()?;
  if token.is_empty() { None } else { Some(token.to_string()) }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn ffmpeg_keeps_the_version_token() {
    let stdout = b"ffmpeg version 7.1.1-essentials_build-www.gyan.dev Copyright (c) 2000-2025\nbuilt with gcc\n";
    assert_eq!(
      version_from_output("ffmpeg", stdout, b""),
      "7.1.1-essentials_build-www.gyan.dev"
    );
  }

  #[test]
  fn ffmpeg_reads_stderr_when_stdout_is_empty() {
    let stderr = b"ffmpeg version N-121157-gff444b24d6-20251015 Copyright\n";
    assert_eq!(version_from_output("ffmpeg", b" \n", stderr), "N-121157-gff444b24d6-20251015");
  }

  #[test]
  fn ffmpeg_unknown_when_banner_has_no_version() {
    assert_eq!(version_from_output("ffmpeg", b"configuration: --enable-gpl\n", b""), UNKNOWN);
  }

  #[test]
  fn cjxl_keeps_the_first_line() {
    assert_eq!(
      version_from_output("cjxl", b"cjxl v0.11.1 [AVX2,SSE4]\n", b""),
      "cjxl v0.11.1 [AVX2,SSE4]"
    );
  }

  #[test]
  fn avifenc_keeps_the_version_number() {
    let stdout = b"Version: 1.1.4 (dav1d [dec]:1.5.0, aom [enc/dec]:3.11.0)\n";
    assert_eq!(version_from_output("avifenc", stdout, b""), "1.1.4");
  }

  #[test]
  fn empty_output_is_unavailable() {
    assert_eq!(version_from_output("cjxl", b"", b"\n"), UNAVAILABLE);
  }
}
