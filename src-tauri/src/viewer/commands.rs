use tauri::{AppHandle, Manager};

use super::decode::{self, ViewerDisplay};
use super::error::ViewerError;
use super::folder::{self, ViewerFile, ViewerFolder};
use super::open;

/// 列出图片所在文件夹这一层
///
/// 放在阻塞线程上，避免大文件夹读文件头时卡住界面。
///
/// # 参数
/// - `app`: 应用句柄，用来放行这一层图片的读取
/// - `path`: 图片或文件夹路径
///
/// # 返回值
/// 成功时返回这一层的图片和当前下标
///
/// # 错误
/// 见 [`folder::read`]。线程本身失败时返回 `ViewerError::Unreadable`
///
/// # Panics
/// 不会主动 panic
#[tauri::command]
pub async fn viewer_folder(app: AppHandle, path: String) -> Result<ViewerFolder, ViewerError> {
  let folder =
    tauri::async_runtime::spawn_blocking(move || folder::read(&path)).await.map_err(|_| ViewerError::Unreadable)?;
  if let Ok(folder) = &folder {
    let _ = app.asset_protocol_scope().allow_directory(&folder.directory, false);
  }
  folder
}

/// 把一张图片改成新的文件名
///
/// 放在阻塞线程上，避免磁盘慢时卡住界面。
///
/// # 参数
/// - `app`: 应用句柄，用来放行改名后的文件
/// - `path`: 现在的图片路径
/// - `name`: 新文件名，不含目录
///
/// # 返回值
/// 成功时返回改名后的文件
///
/// # 错误
/// 见 [`folder::rename`]。线程本身失败时返回 `ViewerError::Rename`
///
/// # Panics
/// 不会主动 panic
#[tauri::command]
pub async fn viewer_rename(app: AppHandle, path: String, name: String) -> Result<ViewerFile, ViewerError> {
  let renamed = tauri::async_runtime::spawn_blocking(move || folder::rename(&path, &name))
    .await
    .map_err(|_| ViewerError::Rename)?;
  if let Ok(file) = &renamed {
    let _ = app.asset_protocol_scope().allow_file(&file.path);
  }
  renamed
}

/// 准备一张可以显示的图
///
/// JPEG XL 会在阻塞线程上解码。
///
/// # 参数
/// - `app`: 应用句柄，用来放解码缓存
/// - `path`: 图片路径
///
/// # 返回值
/// 成功时返回显示路径和像素尺寸
///
/// # 错误
/// 见 [`decode::prepare`]。线程本身失败时返回 `ViewerError::Decode`
///
/// # Panics
/// 不会主动 panic
#[tauri::command]
pub async fn viewer_display(app: AppHandle, path: String) -> Result<ViewerDisplay, ViewerError> {
  tauri::async_runtime::spawn_blocking(move || decode::prepare(&app, &path)).await.map_err(|_| ViewerError::Decode)?
}

/// 取走启动参数里暂存的图片路径
///
/// # 参数
/// - `app`: 应用句柄
///
/// # 返回值
/// 有暂存时返回路径，没有时为空
///
/// # Panics
/// 不会主动 panic
#[tauri::command]
pub fn viewer_take_pending(app: AppHandle) -> Option<String> {
  open::take(&app)
}

/// 登记为图片的默认打开方式
///
/// 对话框可能停住一段时间，所以放在阻塞线程上。
///
/// # 返回值
/// 登记完成，或已经打开系统设置
///
/// # 错误
/// 见 [`super::associate::register_default`]
///
/// # Panics
/// 不会主动 panic
#[tauri::command]
pub async fn viewer_register_default() -> Result<(), ViewerError> {
  tauri::async_runtime::spawn_blocking(super::associate::register_default).await.map_err(|_| ViewerError::Settings)?
}
