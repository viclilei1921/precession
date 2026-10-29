// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// 启动应用
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
  precession_lib::run()
}
