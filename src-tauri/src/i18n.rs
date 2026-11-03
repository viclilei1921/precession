//! 后端需要自己显示的文字。
//!
//! 前端通过 `set_app_locale` 把当前语言同步过来。托盘菜单等原生 UI 在切换语言后重建。
//! 业务错误仍以稳定文本返回，由前端 `errors` 语言包翻译。

use std::sync::atomic::{AtomicU8, Ordering};

/// 0 = 简体中文，1 = English
static LOCALE: AtomicU8 = AtomicU8::new(0);

/// 记录前端当前语言。`zh-*` 归为中文，其余归为英文。
pub fn set(language: &str) {
  let next = if language.starts_with("zh") { 0 } else { 1 };
  LOCALE.store(next, Ordering::Relaxed);
}

/// 当前是否为英文
pub fn is_english() -> bool {
  LOCALE.load(Ordering::Relaxed) == 1
}

/// 托盘菜单文字
///
/// # 参数
/// - `key`: `quit` 或 `show`
///
/// # 返回值
/// 当前语言下的菜单文字；未知 key 原样返回
pub fn tray(key: &str) -> &'static str {
  match (is_english(), key) {
    (false, "quit") => "退出",
    (false, "show") => "显示窗口",
    (true, "quit") => "Quit",
    (true, "show") => "Show Window",
    _ => "",
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn locale_switch_changes_tray_text() {
    set("zh-CN");
    assert_eq!(tray("quit"), "退出");
    set("en-US");
    assert_eq!(tray("quit"), "Quit");
    assert_eq!(tray("show"), "Show Window");
    set("zh-CN");
  }
}
