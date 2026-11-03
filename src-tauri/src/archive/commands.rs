use std::path::Path;
#[cfg(target_os = "android")]
use std::path::PathBuf;

#[cfg(target_os = "android")]
use tauri::Manager;
use tauri::{AppHandle, State};

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

/// 把档案写进应用缓存，并打开系统分享
///
/// 只在 Android 上分享。桌面端仍走 `archive_export` 和保存对话框。
///
/// # 参数
/// - `app`: 用来取缓存目录和设备槽
/// - `db`: 进程内数据库句柄
/// - `queue`: 任务队列
/// - `name`: 压缩包文件名，不能带目录
///
/// # 返回值
/// 成功时返回缓存路径、文件个数和字节数
///
/// # 错误
/// 文件名不合法、缓存目录不可用、打包失败或分享打不开时返回对应的 `ArchiveError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let packed = archive_export_share(app, db, queue, name)?;
/// ```
#[tauri::command]
pub fn archive_export_share(
  app: AppHandle,
  db: State<'_, DbState>,
  queue: State<'_, TaskQueue>,
  name: String,
) -> Result<ArchiveExport, ArchiveError> {
  export_then_share(app, db, queue, name)
}

/// Android 上写入缓存并分享。其他平台拒绝
#[cfg(target_os = "android")]
fn export_then_share(
  app: AppHandle,
  db: State<'_, DbState>,
  queue: State<'_, TaskQueue>,
  name: String,
) -> Result<ArchiveExport, ArchiveError> {
  let dest = cache_dest(&app, &name)?;
  let packed = export_archive(&db, &queue, &dest)?;
  present_share(&app, &packed.path)?;
  Ok(packed)
}

/// 桌面端不从这里分享
#[cfg(not(target_os = "android"))]
fn export_then_share(
  app: AppHandle,
  db: State<'_, DbState>,
  queue: State<'_, TaskQueue>,
  name: String,
) -> Result<ArchiveExport, ArchiveError> {
  let _ = (app, db, queue, name);
  Err(ArchiveError::Share)
}

/// 把压缩包文件名拼到应用缓存目录
///
/// # 参数
/// - `app`: 用来取缓存目录
/// - `name`: 文件名
///
/// # 返回值
/// 缓存目录下的绝对路径
///
/// # 错误
/// 文件名含目录、缓存目录取不到或建不出来时返回 `PathInvalid` 或 `Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let dest = cache_dest(&app, "precession.zip")?;
/// ```
#[cfg(target_os = "android")]
fn cache_dest(app: &AppHandle, name: &str) -> Result<PathBuf, ArchiveError> {
  if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") {
    return Err(ArchiveError::PathInvalid);
  }
  let cache = app.path().app_cache_dir().map_err(|_| ArchiveError::Io)?;
  std::fs::create_dir_all(&cache).map_err(|_| ArchiveError::Io)?;
  Ok(cache.join(name))
}

/// 打开系统分享。非 Android 直接拒绝
///
/// # 参数
/// - `app`: 已挂上设备槽的应用
/// - `path`: 缓存里的压缩包
///
/// # 返回值
/// 分享面板打开后不返回数据
///
/// # 错误
/// 非 Android，或设备槽分享失败时返回 `Share`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// present_share(&app, &packed.path)?;
/// ```
#[cfg(target_os = "android")]
fn present_share(app: &AppHandle, path: &str) -> Result<(), ArchiveError> {
  let api = app.state::<precession_device_slot::DeviceApi<tauri::Wry>>();
  api.share(path).map_err(|_| ArchiveError::Share)
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
