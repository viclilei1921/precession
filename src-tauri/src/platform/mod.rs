//! 各平台启动时挂上自己的插件和窗口行为。不支持的挂钩用默认空实现。

// 始终挂上模块，文件才属于 crate。Android 专用依赖放在文件内部的 cfg 里。
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
mod android;
// 始终挂上模块，文件才属于 crate。macOS 窗口代码放在文件内部的 cfg 里。
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod macos;
// 始终挂上模块，文件才属于 crate。Windows 的侧车和托盘放在文件内部的 cfg 里。
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
mod windows;
// 空实现没有平台依赖，各目标都编进来，避免 rust-analyzer 把该文件当成未归属源文件。
#[cfg_attr(any(desktop, target_os = "android"), allow(dead_code))]
mod unsupported;

use tauri::{App, Webview, Window};

use crate::db::state::DbState;

/// 当前平台的启动挂钩
///
/// 方法都不拿 `self`，编译期选定一个实现。默认方法是空操作。
pub(crate) trait Platform {
  /// 原样返回传入的构建器
  ///
  /// # 参数
  /// - `builder`: 待挂插件的 Tauri 构建器
  ///
  /// # 返回值
  /// 未经修改的构建器
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let builder = Platform::plugins(builder);
  /// ```
  fn plugins(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder
  }

  /// 原样返回传入的数据库句柄
  ///
  /// # 参数
  /// - `app`: 已启动的应用，默认实现不使用
  /// - `db`: 数据库句柄
  ///
  /// # 返回值
  /// 传入的 `db`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let db = Platform::attach_device(app, db);
  /// ```
  fn attach_device(app: &App, db: DbState) -> DbState {
    let _ = app;
    db
  }

  /// 忽略启动阶段的应用参数
  ///
  /// # 参数
  /// - `app`: 已启动的应用，默认实现不使用
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// Platform::setup(app);
  /// ```
  fn setup(app: &mut App) {
    let _ = app;
  }

  /// 忽略页面加载事件
  ///
  /// # 参数
  /// - `webview`: 发生加载的 WebView
  /// - `payload`: 加载事件内容
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// Platform::on_page_load(webview, payload);
  /// ```
  fn on_page_load(webview: &Webview, payload: &tauri::webview::PageLoadPayload<'_>) {
    let _ = (webview, payload);
  }

  /// 忽略窗口事件
  ///
  /// 默认实现不拦截关闭。
  ///
  /// # 参数
  /// - `window`: 发出事件的窗口
  /// - `event`: 窗口事件
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// Platform::on_window_event(window, event);
  /// ```
  fn on_window_event(window: &Window, event: &tauri::WindowEvent) {
    let _ = (window, event);
  }
}

#[cfg(target_os = "android")]
pub(crate) use android::Android as Current;
#[cfg(target_os = "macos")]
pub(crate) use macos::Macos as Current;
#[cfg(not(any(target_os = "android", target_os = "macos", target_os = "windows")))]
pub(crate) use unsupported::Unsupported as Current;
#[cfg(target_os = "windows")]
pub(crate) use windows::Windows as Current;
