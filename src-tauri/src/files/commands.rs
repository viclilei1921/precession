use serde::Serialize;

use super::collect::{self, Collected};
use super::error::FilesError;

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
