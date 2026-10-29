use std::path::Path;

use tauri::State;

use super::dto::{ArchiveExport, ArchiveInfo};
use super::error::ArchiveError;
use super::service::{archive_info as read_info, export_archive, import_archive};
use crate::db::state::DbState;
use crate::task::TaskQueue;
use crate::upload::media::MediaState;

/// 返回数据库路径和上传文件用量
///
/// # 参数
/// - `db`: 进程内数据库句柄
///
/// # 返回值
/// 成功时返回路径、文件个数和字节数
///
/// # 错误
/// 数据库未解锁时返回 `Locked`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let info = archive_info(db)?;
/// ```
#[tauri::command]
pub fn archive_info(db: State<'_, DbState>) -> Result<ArchiveInfo, ArchiveError> {
  read_info(&db)
}

/// 把数据库和上传文件写到用户选择的位置
///
/// # 参数
/// - `db`: 进程内数据库句柄
/// - `queue`: 任务队列
/// - `path`: 保存路径
///
/// # 返回值
/// 成功时返回压缩包路径、文件个数和字节数
///
/// # 错误
/// `export_archive` 返回错误时原样传出该 `ArchiveError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let packed = archive_export(db, queue, path)?;
/// ```
#[tauri::command]
pub fn archive_export(
  db: State<'_, DbState>,
  queue: State<'_, TaskQueue>,
  path: String,
) -> Result<ArchiveExport, ArchiveError> {
  export_archive(&db, &queue, Path::new(&path))
}

/// 用压缩包替换当前数据库和上传文件
///
/// 成功后清掉预览留下的文件密钥。会话已经释放，需要用这份档案的密码重新打开。
///
/// # 参数
/// - `db`: 进程内数据库句柄
/// - `queue`: 任务队列
/// - `media`: 播放地址和文件密钥
/// - `path`: 压缩包路径
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// `import_archive` 返回错误时原样传出该 `ArchiveError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// archive_import(db, queue, media, path)?;
/// ```
#[tauri::command]
pub fn archive_import(
  db: State<'_, DbState>,
  queue: State<'_, TaskQueue>,
  media: State<'_, MediaState>,
  path: String,
) -> Result<(), ArchiveError> {
  import_archive(&db, &queue, Path::new(&path))?;
  media.clear();
  Ok(())
}
