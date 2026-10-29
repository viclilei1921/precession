use crate::utils::time::now_utc8;

const LOG_FILE: &str = "precession";
use tauri::Runtime;
use tauri::plugin::TauriPlugin;
use tauri_plugin_log::{Target, TargetKind, WEBVIEW_TARGET};

/// 创建写到标准输出、日志目录和 WebView 的日志插件
///
/// 每行时间用 UTC+8。来自 WebView 的目标记为 `frontend`，级别从 Info 起。
///
/// # 返回值
/// 构建好的 `TauriPlugin<R>`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let plugin = init_log::<tauri::Wry>();
/// ```
pub fn init_log<R: Runtime>() -> TauriPlugin<R> {
  tauri_plugin_log::Builder::new()
    .targets([
      Target::new(TargetKind::Stdout),
      Target::new(TargetKind::LogDir { file_name: Some(LOG_FILE.into()) }),
      Target::new(TargetKind::Webview),
    ])
    .format(|out, message, record| {
      let (year, month, day, hour, minute, second) = now_utc8();
      let target = if record.target().starts_with(WEBVIEW_TARGET) { "frontend" } else { record.target() };
      out.finish(format_args!(
        "[{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}] [{}] [{target}] {message}",
        record.level(),
      ))
    })
    .level(tauri_plugin_log::log::LevelFilter::Info)
    .build()
}
