use super::collect::{self, Collected};
use super::error::FilesError;

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
