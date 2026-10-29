const COMMANDS: &[&str] = &["enroll", "open", "forget", "confirm"];

/// 用 enroll、open、forget、confirm 构建 Android 插件
///
/// 插件工程路径为 `android`。
///
/// # 返回值
/// 不返回数据
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// main();
/// ```
fn main() {
  tauri_plugin::Builder::new(COMMANDS).android_path("android").build();
}
