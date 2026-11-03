use serde::Serialize;
use tauri::AppHandle;

use super::collect::{self, Collected};
use super::error::FilesError;
use super::materialize::{self, Picked};

/// 一个文件的字节大小
///
/// 读不到的路径不会出现在结果里。
///
/// # 字段
/// - `path`: 文件路径
/// - `size`: 字节数
///
/// # 示例
/// 见 [`file_sizes`]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSize {
  /// 文件路径
  pub path: String,
  /// 字节数
  pub size: u64,
}

/// 收集路径里符合扩展名的文件
///
/// 文件直接按扩展名判断。目录会进入子目录，跳过符号链接和读失败的子目录。扩展名为空时接受任意文件。
///
/// # 参数
/// - `paths`: 文件或目录路径
/// - `extensions`: 不带点的扩展名。空则不过滤
///
/// # 返回值
/// 成功时返回文件路径，并标明是否展开过目录
///
/// # 错误
/// 路径不存在返回 `FilesError::Missing`。路径读不了返回 `FilesError::Unreadable`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let found = collect_files(paths, extensions)?;
/// ```
#[tauri::command]
pub fn collect_files(paths: Vec<String>, extensions: Vec<String>) -> Result<Collected, FilesError> {
  collect::collect(&paths, &extensions)
}

/// 把选择器返回的地址收成可以打开的本地路径
///
/// 桌面路径原样返回。Android 的 `content:` 地址复制进应用缓存后再返回缓存路径。`file:` 地址解开后，文件存在才返回本地路径。
///
/// # 参数
/// - `app`: 应用句柄。Android 上用来调用设备槽复制文件
/// - `paths`: 选择器返回的路径
///
/// # 返回值
/// 与输入顺序相同的本地路径
///
/// # 错误
/// 路径空白或 `file:` 指向的文件不存在时返回 `FilesError::Missing`。content 地址复制失败时返回 `FilesError::Unreadable`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let paths = materialize_picked(app, paths)?;
/// ```
#[tauri::command]
pub fn materialize_picked(app: AppHandle, paths: Vec<String>) -> Result<Vec<String>, FilesError> {
  paths.into_iter().map(|path| materialize_one(&app, &path)).collect()
}

/// 收成一条本地路径
///
/// # 参数
/// - `app`: 应用句柄
/// - `path`: 选择器返回的一条路径
///
/// # 返回值
/// 可以交给 `File::open` 的路径
///
/// # 错误
/// [`materialize::classify`] 失败时原样传出。content 地址复制失败时返回 `FilesError::Unreadable`
///
/// # Panics
/// 不会主动 panic
fn materialize_one(app: &AppHandle, path: &str) -> Result<String, FilesError> {
  match materialize::classify(path)? {
    Picked::Ready(path) => Ok(path),
    Picked::Content(uri) => copy_content(app, &uri),
  }
}

/// 把 Android content 地址复制进缓存
///
/// # 参数
/// - `app`: 已挂上设备槽的应用
/// - `uri`: `content:` 地址
///
/// # 返回值
/// 缓存文件路径
///
/// # 错误
/// 非 Android，或复制失败、结果不是文件时返回 `FilesError::Unreadable`
///
/// # Panics
/// 不会主动 panic
#[cfg(target_os = "android")]
fn copy_content(app: &AppHandle, uri: &str) -> Result<String, FilesError> {
  use tauri::Manager;

  let api = app.state::<precession_device_slot::DeviceApi<tauri::Wry>>();
  let copied = api.copy_picked(uri).map_err(|_| FilesError::Unreadable)?;
  if std::path::Path::new(&copied).is_file() { Ok(copied) } else { Err(FilesError::Unreadable) }
}

/// 桌面端没有 content 地址可复制
#[cfg(not(target_os = "android"))]
fn copy_content(_app: &AppHandle, _uri: &str) -> Result<String, FilesError> {
  Err(FilesError::Unreadable)
}

/// 读取文件的字节大小
///
/// 目录、不存在或读失败的路径会被跳过。
///
/// # 参数
/// - `paths`: 文件路径
///
/// # 返回值
/// 能读到的文件和它的字节数
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let sizes = file_sizes(vec!["D:\\a.txt".into()]);
/// ```
#[tauri::command]
pub fn file_sizes(paths: Vec<String>) -> Vec<FileSize> {
  paths
    .into_iter()
    .filter_map(|path| {
      let meta = std::fs::metadata(&path).ok()?;
      if !meta.is_file() {
        return None;
      }
      Some(FileSize { path, size: meta.len() })
    })
    .collect()
}
