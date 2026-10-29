//! 把数据库和上传文件打成一个压缩包，或从压缩包整份换上。
//!
//! 包里有 `manifest.json`、`db/data.sqlite`、`db/key.header.json` 和 `upload/` 下的正式文件。
//! 设备槽和用户锁定标记不进包。导入后会关掉设备解锁，并清掉内存里的会话。

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

use rusqlite::Connection;
use serde_json::Value;
use zip::result::ZipError;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use super::dto::{ArchiveExport, ArchiveInfo};
use super::error::ArchiveError;
use crate::db::state::DbState;
use crate::db::{DB_DEVICE_WRAP_TMP, DB_DIR, DB_HEADER, DB_SQLITE, DB_USER_LOCK, read_header};
use crate::task::TaskQueue;
use crate::upload::{TMP_DIR, UPLOAD_DIR};
use crate::utils::id::new_uuid_v4;

const FORMAT: &str = "precession-archive";
const VERSION: u64 = 1;
const MANIFEST: &str = "manifest.json";

/// 临时目录
///
/// 丢弃时删掉整棵目录。
///
/// # 字段
/// - `0`: 目录路径
///
/// # 示例
/// ```ignore
/// let dir = TempDir::new()?;
/// ```
struct TempDir(
  /// 目录路径
  PathBuf,
);

impl TempDir {
  /// 在系统临时目录下建一个空目录
  ///
  /// # 返回值
  /// 成功时返回会在丢弃时删除的目录
  ///
  /// # 错误
  /// 创建目录失败时返回 `Io`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let dir = TempDir::new()?;
  /// ```
  fn new() -> Result<Self, ArchiveError> {
    let path = std::env::temp_dir().join(format!("precession-{uuid}", uuid = new_uuid_v4()));
    fs::create_dir_all(&path).map_err(io_err)?;
    Ok(Self(path))
  }

  /// 返回临时目录路径
  ///
  /// # 返回值
  /// 目录路径
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let path = dir.path();
  /// ```
  fn path(&self) -> &Path {
    &self.0
  }
}

impl Drop for TempDir {
  /// 删掉临时目录
  ///
  /// 删除失败时忽略。
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// drop(dir);
  /// ```
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

/// 读取数据库路径和上传文件用量
///
/// 数据库未解锁时拒绝。用量来自磁盘上的正式文件。
///
/// # 参数
/// - `db`: 数据库状态
///
/// # 返回值
/// 路径、文件个数和字节数
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
pub(crate) fn archive_info(db: &DbState) -> Result<ArchiveInfo, ArchiveError> {
  ensure_unlocked(db)?;
  let usage = upload_usage(&data_root(db));
  Ok(ArchiveInfo { path: path_text(&db.sqlite), file_count: usage.0, file_bytes: usage.1 })
}

/// 把数据库和上传文件写到用户选择的压缩包
///
/// 先把库真空复制出来，再连同密钥头和正式文件写入压缩包。保存位置不能落在数据目录里面。
/// 还有任务在处理时拒绝。
///
/// # 参数
/// - `db`: 数据库状态
/// - `queue`: 任务队列
/// - `dest`: 用户选择的保存路径
///
/// # 返回值
/// 压缩包路径、打进包里的文件个数和压缩包字节数
///
/// # 错误
/// - 路径为空返回 `PathEmpty`，落在数据目录里返回 `PathInvalid`
/// - 数据库未解锁返回 `Locked`，库文件不在返回 `Missing`
/// - 还有任务在处理返回 `Busy`
/// - 复制库或写压缩包失败返回 `Io` 或 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let packed = export_archive(db, queue, dest)?;
/// ```
pub(crate) fn export_archive(db: &DbState, queue: &TaskQueue, dest: &Path) -> Result<ArchiveExport, ArchiveError> {
  let dest = checked_dest(db, dest)?;
  ensure_unlocked(db)?;
  ensure_idle(queue)?;
  if !db.sqlite.is_file() || !db.header.is_file() {
    return Err(ArchiveError::Missing);
  }

  let scratch = TempDir::new()?;
  let snapshot = scratch.path().join(DB_SQLITE);
  db.with_conn(|conn| vacuum_into(conn, &snapshot))?;
  let files = upload_files(&data_root(db));
  let partial = partial_path(&dest);
  write_zip(&partial, &snapshot, &db.header, &data_root(db), &files)?;
  if dest.exists() {
    fs::remove_file(&dest).map_err(io_err)?;
  }
  fs::rename(&partial, &dest).map_err(io_err)?;
  let bytes = fs::metadata(&dest).map(|meta| meta.len()).unwrap_or(0);
  Ok(ArchiveExport { path: path_text(&dest), file_count: files.len() as u64, bytes })
}

/// 用压缩包替换当前数据库和上传文件
///
/// 校验通过后清掉会话，换上包里的库、密钥头和文件，再关闭设备解锁。
/// 失败时尽量把原来的库和文件移回去。导入后需要用这份档案的密码重新打开。
///
/// # 参数
/// - `db`: 数据库状态
/// - `queue`: 任务队列
/// - `src`: 用户选择的压缩包
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// - 路径为空返回 `PathEmpty`，压缩包在数据目录里返回 `PathInvalid`
/// - 数据库未解锁返回 `Locked`
/// - 还有任务在处理返回 `Busy`
/// - 压缩包格式不对返回 `Invalid`
/// - 换文件失败返回 `Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// import_archive(db, queue, src)?;
/// ```
pub(crate) fn import_archive(db: &DbState, queue: &TaskQueue, src: &Path) -> Result<(), ArchiveError> {
  if src.as_os_str().is_empty() {
    return Err(ArchiveError::PathEmpty);
  }
  if !src.is_file() {
    return Err(ArchiveError::Missing);
  }
  if inside_data(db, src) {
    return Err(ArchiveError::PathInvalid);
  }
  ensure_unlocked(db)?;
  ensure_idle(queue)?;

  let staged = TempDir::new()?;
  extract_archive(src, staged.path())?;
  read_header(&staged.path().join(DB_DIR).join(DB_HEADER)).map_err(|_| ArchiveError::Invalid)?;

  ensure_idle(queue)?;
  db.replace_unlocked(|| {
    remove_sidecars(db);
    swap_in(db, staged.path())
  })?;
  finish_import(db);
  Ok(())
}

/// 确认数据库已经解锁
///
/// # 参数
/// - `db`: 数据库状态
///
/// # 返回值
/// 已解锁时不返回数据
///
/// # 错误
/// 未解锁时返回 `Locked`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// ensure_unlocked(db)?;
/// ```
fn ensure_unlocked(db: &DbState) -> Result<(), ArchiveError> {
  if db.status().unlocked { Ok(()) } else { Err(ArchiveError::Locked) }
}

/// 确认没有待处理或处理中的任务
///
/// # 参数
/// - `queue`: 任务队列
///
/// # 返回值
/// 队列空闲时不返回数据
///
/// # 错误
/// 有任务在处理时返回 `Busy`，读取队列失败时返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// ensure_idle(queue)?;
/// ```
fn ensure_idle(queue: &TaskQueue) -> Result<(), ArchiveError> {
  if queue.busy().map_err(|_| ArchiveError::Internal)? { Err(ArchiveError::Busy) } else { Ok(()) }
}

/// 检查导出路径，并拒绝写进数据目录
///
/// # 参数
/// - `db`: 数据库状态
/// - `dest`: 用户选择的保存路径
///
/// # 返回值
/// 可用的保存路径
///
/// # 错误
/// 路径为空返回 `PathEmpty`，落在数据目录里或父目录不存在返回 `PathInvalid`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let dest = checked_dest(db, path)?;
/// ```
fn checked_dest(db: &DbState, dest: &Path) -> Result<PathBuf, ArchiveError> {
  if dest.as_os_str().is_empty() {
    return Err(ArchiveError::PathEmpty);
  }
  let parent = dest.parent().filter(|path| !path.as_os_str().is_empty()).ok_or(ArchiveError::PathInvalid)?;
  if !parent.is_dir() {
    return Err(ArchiveError::PathInvalid);
  }
  if inside_data(db, dest) {
    return Err(ArchiveError::PathInvalid);
  }
  Ok(dest.to_path_buf())
}

/// 判断路径是不是在数据目录里面
///
/// 目标还不存在时，用已经存在的父目录来比较。
///
/// # 参数
/// - `db`: 数据库状态
/// - `path`: 要检查的路径
///
/// # 返回值
/// 位于数据目录内时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let blocked = inside_data(db, dest);
/// ```
fn inside_data(db: &DbState, path: &Path) -> bool {
  let root = normalize(&data_root(db));
  let target = normalize(path);
  target.starts_with(&root)
}

/// 尽量解析成绝对路径
///
/// 文件还不存在时，解析它的父目录再接上文件名。
///
/// # 参数
/// - `path`: 原路径
///
/// # 返回值
/// 解析后的路径。解析失败时返回原路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let full = normalize(path);
/// ```
fn normalize(path: &Path) -> PathBuf {
  if let Ok(full) = path.canonicalize() {
    return full;
  }
  let Some(name) = path.file_name() else {
    return path.to_path_buf();
  };
  let Some(parent) = path.parent() else {
    return path.to_path_buf();
  };
  parent.canonicalize().map(|dir| dir.join(name)).unwrap_or_else(|_| path.to_path_buf())
}

/// 取数据库目录的父目录
///
/// # 参数
/// - `db`: 数据库状态
///
/// # 返回值
/// 数据根目录
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let root = data_root(db);
/// ```
fn data_root(db: &DbState) -> PathBuf {
  db.dir.parent().map(Path::to_path_buf).unwrap_or_else(|| db.dir.clone())
}

/// 把路径显示成字符串
///
/// # 参数
/// - `path`: 路径
///
/// # 返回值
/// 有损转换后的文本
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let text = path_text(path);
/// ```
fn path_text(path: &Path) -> String {
  path.to_string_lossy().to_string()
}

/// 统计上传目录里的正式文件
///
/// 跳过临时目录和符号链接。目录读失败时略过这一支。
///
/// # 参数
/// - `root`: 数据根目录
///
/// # 返回值
/// 文件个数和字节数
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let (count, bytes) = upload_usage(root);
/// ```
fn upload_usage(root: &Path) -> (u64, u64) {
  let files = upload_files(root);
  let bytes = files.iter().filter_map(|path| fs::metadata(path).ok()).map(|meta| meta.len()).sum();
  (files.len() as u64, bytes)
}

/// 列出上传目录里要打包的文件
///
/// # 参数
/// - `root`: 数据根目录
///
/// # 返回值
/// 正式文件的绝对路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let files = upload_files(root);
/// ```
fn upload_files(root: &Path) -> Vec<PathBuf> {
  let upload = root.join(UPLOAD_DIR);
  let tmp = upload.join(TMP_DIR);
  let mut found = Vec::new();
  let mut pending = vec![upload];
  while let Some(dir) = pending.pop() {
    if dir == tmp {
      continue;
    }
    let Ok(entries) = fs::read_dir(&dir) else {
      continue;
    };
    for entry in entries.flatten() {
      let path = entry.path();
      if path == tmp || path.starts_with(&tmp) {
        continue;
      }
      let Ok(kind) = entry.file_type() else {
        continue;
      };
      if kind.is_symlink() {
        continue;
      }
      if kind.is_dir() {
        pending.push(path);
        continue;
      }
      if kind.is_file() {
        found.push(path);
      }
    }
  }
  found.sort();
  found
}

/// 把当前库真空复制到一个新文件
///
/// # 参数
/// - `conn`: 已解锁的连接
/// - `dest`: 复制结果的路径
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 复制失败时返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// vacuum_into(conn, dest)?;
/// ```
fn vacuum_into(conn: &Connection, dest: &Path) -> Result<(), ArchiveError> {
  let text = dest.to_str().ok_or(ArchiveError::Io)?.replace('\'', "''");
  conn.execute(&format!("VACUUM INTO '{text}'"), []).map_err(|err| {
    tauri_plugin_log::log::error!("archive vacuum: {err}");
    ArchiveError::Internal
  })?;
  Ok(())
}

/// 写出压缩包。写到一半失败时删掉半成品
///
/// # 参数
/// - `dest`: 半成品路径
/// - `sqlite`: 真空复制出的库
/// - `header`: 密钥头
/// - `root`: 数据根目录
/// - `files`: 要写入的上传文件
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 写压缩包失败时返回 `Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// write_zip(partial, sqlite, header, root, &files)?;
/// ```
fn write_zip(dest: &Path, sqlite: &Path, header: &Path, root: &Path, files: &[PathBuf]) -> Result<(), ArchiveError> {
  let file = File::create(dest).map_err(io_err)?;
  let result = write_entries(file, sqlite, header, root, files);
  if result.is_err() {
    let _ = fs::remove_file(dest);
  }
  result
}

/// 把清单、库、密钥头和上传文件写进已经打开的压缩包
///
/// # 参数
/// - `file`: 压缩包文件
/// - `sqlite`: 真空复制出的库
/// - `header`: 密钥头
/// - `root`: 数据根目录
/// - `files`: 要写入的上传文件
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 写入失败时返回 `Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// write_entries(file, sqlite, header, root, &files)?;
/// ```
fn write_entries(file: File, sqlite: &Path, header: &Path, root: &Path, files: &[PathBuf]) -> Result<(), ArchiveError> {
  let mut zip = ZipWriter::new(file);
  let manifest = serde_json::json!({ "format": FORMAT, "version": VERSION });
  let body = serde_json::to_vec(&manifest).map_err(|_| ArchiveError::Internal)?;
  add_bytes(&mut zip, MANIFEST, &body)?;
  add_path(&mut zip, &format!("{DB_DIR}/{DB_SQLITE}"), sqlite)?;
  add_path(&mut zip, &format!("{DB_DIR}/{DB_HEADER}"), header)?;
  for path in files {
    let Some(name) = zip_name(root, path) else {
      continue;
    };
    add_path(&mut zip, &name, path)?;
  }
  zip.finish().map_err(zip_err)?;
  Ok(())
}

/// 把内存里的字节写成一个压缩包条目
///
/// # 参数
/// - `zip`: 压缩包写入器
/// - `name`: 条目名
/// - `bytes`: 条目内容
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 写入失败时返回 `Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// add_bytes(&mut zip, "manifest.json", bytes)?;
/// ```
fn add_bytes(zip: &mut ZipWriter<File>, name: &str, bytes: &[u8]) -> Result<(), ArchiveError> {
  zip.start_file(name, zip_options()).map_err(zip_err)?;
  zip.write_all(bytes).map_err(io_err)?;
  Ok(())
}

/// 把一个磁盘文件写成一个压缩包条目
///
/// # 参数
/// - `zip`: 压缩包写入器
/// - `name`: 条目名
/// - `path`: 源文件
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 读取或写入失败时返回 `Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// add_path(&mut zip, "db/data.sqlite", path)?;
/// ```
fn add_path(zip: &mut ZipWriter<File>, name: &str, path: &Path) -> Result<(), ArchiveError> {
  let mut src = File::open(path).map_err(io_err)?;
  zip.start_file(name, zip_options()).map_err(zip_err)?;
  io::copy(&mut src, zip).map_err(io_err)?;
  Ok(())
}

/// 压缩包条目的写入选项
///
/// 不压缩。照片和视频再压一次几乎不变小，只会更慢。
///
/// # 返回值
/// 存储方式的条目选项
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let options = zip_options();
/// ```
fn zip_options() -> SimpleFileOptions {
  SimpleFileOptions::default().compression_method(CompressionMethod::Stored)
}

/// 把文件路径换成压缩包里的相对名字
///
/// # 参数
/// - `root`: 数据根目录
/// - `path`: 文件绝对路径
///
/// # 返回值
/// 使用 `/` 的相对路径。不在根目录下时为 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let name = zip_name(root, path);
/// ```
fn zip_name(root: &Path, path: &Path) -> Option<String> {
  let rel = path.strip_prefix(root).ok()?;
  let mut parts = Vec::new();
  for part in rel.components() {
    match part {
      Component::Normal(text) => parts.push(text.to_string_lossy().to_string()),
      _ => return None,
    }
  }
  if parts.is_empty() { None } else { Some(parts.join("/")) }
}

/// 半成品压缩包的路径，和目标文件放在同一目录
///
/// # 参数
/// - `dest`: 最终保存路径
///
/// # 返回值
/// 文件名后面加 `.partial` 的路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let partial = partial_path(dest);
/// ```
fn partial_path(dest: &Path) -> PathBuf {
  let name =
    dest.file_name().map(|text| text.to_string_lossy().to_string()).unwrap_or_else(|| "archive.zip".to_string());
  dest.with_file_name(format!("{name}.partial"))
}

/// 把压缩包解到临时目录，并核对清单
///
/// 拒绝绝对路径和 `..`。只留下清单、库、密钥头和 `upload/` 里的文件。
///
/// # 参数
/// - `src`: 压缩包路径
/// - `dest`: 解压目录
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 打不开、格式不对或写出失败时返回 `Invalid` 或 `Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// extract_archive(src, dest)?;
/// ```
fn extract_archive(src: &Path, dest: &Path) -> Result<(), ArchiveError> {
  let file = File::open(src).map_err(io_err)?;
  let mut archive = ZipArchive::new(file).map_err(|_| ArchiveError::Invalid)?;
  let mut saw_manifest = false;
  let mut saw_sqlite = false;
  let mut saw_header = false;
  for index in 0..archive.len() {
    let mut entry = archive.by_index(index).map_err(|_| ArchiveError::Invalid)?;
    if entry.is_dir() {
      continue;
    }
    let raw = entry.name().map_err(|_| ArchiveError::Invalid)?;
    let name = raw.replace('\\', "/");
    let Some(rel) = safe_rel(&name) else {
      return Err(ArchiveError::Invalid);
    };
    let kind = entry_kind(&rel);
    if kind == EntryKind::Skip {
      continue;
    }
    let target = dest.join(&rel);
    if let Some(parent) = target.parent() {
      fs::create_dir_all(parent).map_err(io_err)?;
    }
    let mut out = File::create(&target).map_err(io_err)?;
    io::copy(&mut entry, &mut out).map_err(io_err)?;
    match kind {
      EntryKind::Manifest => saw_manifest = true,
      EntryKind::Sqlite => saw_sqlite = true,
      EntryKind::Header => saw_header = true,
      EntryKind::Upload | EntryKind::Skip => {}
    }
  }
  if !saw_manifest || !saw_sqlite || !saw_header {
    return Err(ArchiveError::Invalid);
  }
  check_manifest(&dest.join(MANIFEST))
}

/// 压缩包条目的用途
///
/// # 变体
/// - `Manifest`: 清单
/// - `Sqlite`: 数据库文件
/// - `Header`: 密钥头
/// - `Upload`: 上传文件
/// - `Skip`: 可以忽略的其他条目
#[derive(Clone, Copy, PartialEq, Eq)]
enum EntryKind {
  /// 清单
  Manifest,
  /// 数据库文件
  Sqlite,
  /// 密钥头
  Header,
  /// 上传文件
  Upload,
  /// 可以忽略的其他条目
  Skip,
}

/// 判断条目该留下还是跳过
///
/// # 参数
/// - `rel`: 已经校验过的相对路径
///
/// # 返回值
/// 条目用途。不认识的路径为 `Skip`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let kind = entry_kind(Path::new("db/data.sqlite"));
/// ```
fn entry_kind(rel: &Path) -> EntryKind {
  let text = rel.components().map(|part| part.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/");
  if text == MANIFEST {
    return EntryKind::Manifest;
  }
  if text == format!("{DB_DIR}/{DB_SQLITE}") {
    return EntryKind::Sqlite;
  }
  if text == format!("{DB_DIR}/{DB_HEADER}") {
    return EntryKind::Header;
  }
  let upload = format!("{UPLOAD_DIR}/");
  let tmp = format!("{UPLOAD_DIR}/{TMP_DIR}");
  if text.starts_with(&upload) && text != tmp && !text.starts_with(&format!("{tmp}/")) {
    return EntryKind::Upload;
  }
  EntryKind::Skip
}

/// 只接受普通相对路径
///
/// # 参数
/// - `name`: 压缩包条目名
///
/// # 返回值
/// 由普通路径段组成的相对路径。绝对路径或含 `..` 时为 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// assert!(safe_rel("../db/data.sqlite").is_none());
/// assert!(safe_rel("db/data.sqlite").is_some());
/// ```
fn safe_rel(name: &str) -> Option<PathBuf> {
  if name.is_empty() || name.ends_with('/') {
    return None;
  }
  let path = Path::new(name);
  if path.is_absolute() {
    return None;
  }
  let mut out = PathBuf::new();
  for part in path.components() {
    match part {
      Component::Normal(text) => out.push(text),
      _ => return None,
    }
  }
  if out.as_os_str().is_empty() { None } else { Some(out) }
}

/// 核对清单的格式名和版本
///
/// # 参数
/// - `path`: 清单文件
///
/// # 返回值
/// 格式和版本都对时不返回数据
///
/// # 错误
/// 读不到或内容不对时返回 `Invalid`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// check_manifest(path)?;
/// ```
fn check_manifest(path: &Path) -> Result<(), ArchiveError> {
  let file = File::open(path).map_err(|_| ArchiveError::Invalid)?;
  let value: Value = serde_json::from_reader(file).map_err(|_| ArchiveError::Invalid)?;
  let format_ok = value.get("format").and_then(Value::as_str) == Some(FORMAT);
  let version_ok = value.get("version").and_then(Value::as_u64) == Some(VERSION);
  if format_ok && version_ok { Ok(()) } else { Err(ArchiveError::Invalid) }
}

/// 用解压出来的库和文件换掉当前数据
///
/// 先把现有文件移到备份目录。中途失败再移回去。
///
/// # 参数
/// - `db`: 数据库状态
/// - `staged`: 解压目录
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 移动失败时返回 `Io`，并尽量恢复原文件
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// swap_in(db, staged)?;
/// ```
fn swap_in(db: &DbState, staged: &Path) -> Result<(), ArchiveError> {
  let backup = TempDir::new()?;
  let root = data_root(db);
  let upload = root.join(UPLOAD_DIR);
  fs::create_dir_all(&db.dir).map_err(io_err)?;

  let sqlite_bak = backup.path().join(DB_SQLITE);
  let header_bak = backup.path().join(DB_HEADER);
  let upload_bak = backup.path().join(UPLOAD_DIR);
  let moved_sqlite = move_aside(&db.sqlite, &sqlite_bak)?;
  let moved_header = match move_aside(&db.header, &header_bak) {
    Ok(moved) => moved,
    Err(err) => {
      restore(moved_sqlite, &sqlite_bak, &db.sqlite);
      return Err(err);
    }
  };
  let moved_upload = match move_aside(&upload, &upload_bak) {
    Ok(moved) => moved,
    Err(err) => {
      restore(moved_sqlite, &sqlite_bak, &db.sqlite);
      restore(moved_header, &header_bak, &db.header);
      return Err(err);
    }
  };

  let staged_sqlite = staged.join(DB_DIR).join(DB_SQLITE);
  let staged_header = staged.join(DB_DIR).join(DB_HEADER);
  let staged_upload = staged.join(UPLOAD_DIR);
  if let Err(err) = relocate(&staged_sqlite, &db.sqlite) {
    rollback(
      db,
      &upload,
      moved_sqlite,
      &sqlite_bak,
      moved_header,
      &header_bak,
      moved_upload,
      &upload_bak,
    );
    return Err(err);
  }
  if let Err(err) = relocate(&staged_header, &db.header) {
    rollback(
      db,
      &upload,
      moved_sqlite,
      &sqlite_bak,
      moved_header,
      &header_bak,
      moved_upload,
      &upload_bak,
    );
    return Err(err);
  }
  if staged_upload.exists() {
    if let Err(err) = relocate(&staged_upload, &upload) {
      rollback(
        db,
        &upload,
        moved_sqlite,
        &sqlite_bak,
        moved_header,
        &header_bak,
        moved_upload,
        &upload_bak,
      );
      return Err(err);
    }
  }
  Ok(())
}

/// 换入失败时删掉半成品，再把备份移回原处
///
/// # 参数
/// - `db`: 数据库状态
/// - `upload`: 上传目录
/// - `moved_sqlite`: 原来的库是否被移走
/// - `sqlite_bak`: 库的备份路径
/// - `moved_header`: 原来的密钥头是否被移走
/// - `header_bak`: 密钥头的备份路径
/// - `moved_upload`: 原来的上传目录是否被移走
/// - `upload_bak`: 上传目录的备份路径
///
/// # 返回值
/// 不返回数据
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// rollback(db, upload, true, sqlite_bak, true, header_bak, false, upload_bak);
/// ```
fn rollback(
  db: &DbState,
  upload: &Path,
  moved_sqlite: bool,
  sqlite_bak: &Path,
  moved_header: bool,
  header_bak: &Path,
  moved_upload: bool,
  upload_bak: &Path,
) {
  let _ = fs::remove_file(&db.sqlite);
  let _ = fs::remove_file(&db.header);
  let _ = fs::remove_dir_all(upload);
  restore(moved_sqlite, sqlite_bak, &db.sqlite);
  restore(moved_header, header_bak, &db.header);
  restore(moved_upload, upload_bak, upload);
}

/// 把一个备份移回原处
///
/// 没有备份时什么也不做。
///
/// # 参数
/// - `moved`: 是否真的移走过
/// - `from`: 备份路径
/// - `to`: 原来的路径
///
/// # 返回值
/// 不返回数据
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// restore(true, backup, original);
/// ```
fn restore(moved: bool, from: &Path, to: &Path) {
  if moved {
    let _ = relocate(from, to);
  }
}

/// 文件或目录存在时移到备份位置
///
/// # 参数
/// - `from`: 原路径
/// - `to`: 备份路径
///
/// # 返回值
/// 确实移动过时为 `true`
///
/// # 错误
/// 移动失败时返回 `Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let moved = move_aside(src, backup)?;
/// ```
fn move_aside(from: &Path, to: &Path) -> Result<bool, ArchiveError> {
  if !from.exists() {
    return Ok(false);
  }
  relocate(from, to)?;
  Ok(true)
}

/// 移动文件或目录。跨盘时改为复制再删除
///
/// # 参数
/// - `from`: 源路径
/// - `to`: 目标路径
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 移动或复制失败时返回 `Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// relocate(src, dest)?;
/// ```
fn relocate(from: &Path, to: &Path) -> Result<(), ArchiveError> {
  if let Some(parent) = to.parent() {
    fs::create_dir_all(parent).map_err(io_err)?;
  }
  if fs::rename(from, to).is_ok() {
    return Ok(());
  }
  if from.is_dir() {
    copy_dir(from, to)?;
    fs::remove_dir_all(from).map_err(io_err)?;
    return Ok(());
  }
  fs::copy(from, to).map_err(io_err)?;
  fs::remove_file(from).map_err(io_err)?;
  Ok(())
}

/// 递归复制目录
///
/// # 参数
/// - `from`: 源目录
/// - `to`: 目标目录
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 创建目录或复制文件失败时返回 `Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// copy_dir(src, dest)?;
/// ```
fn copy_dir(from: &Path, to: &Path) -> Result<(), ArchiveError> {
  fs::create_dir_all(to).map_err(io_err)?;
  for entry in fs::read_dir(from).map_err(io_err)? {
    let entry = entry.map_err(io_err)?;
    let target = to.join(entry.file_name());
    if entry.path().is_dir() {
      copy_dir(&entry.path(), &target)?;
    } else {
      fs::copy(entry.path(), &target).map_err(io_err)?;
    }
  }
  Ok(())
}

/// 删掉库旁边的日志文件，避免旧日志套到新库上
///
/// # 参数
/// - `db`: 数据库状态
///
/// # 返回值
/// 不返回数据
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// remove_sidecars(db);
/// ```
fn remove_sidecars(db: &DbState) {
  for suffix in ["-wal", "-shm", "-journal"] {
    let mut name = db.sqlite.as_os_str().to_os_string();
    name.push(suffix);
    let _ = fs::remove_file(PathBuf::from(name));
  }
}

/// 关掉设备解锁，并清掉用户锁定标记
///
/// 设备槽删除失败时仍删掉密文文件，避免用旧密钥去开新库。
///
/// # 参数
/// - `db`: 数据库状态
///
/// # 返回值
/// 不返回数据
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// finish_import(db);
/// ```
fn finish_import(db: &DbState) {
  if db.device_wrap.exists() {
    if let Err(err) = db.disable_device() {
      tauri_plugin_log::log::error!("archive import device: {err}");
      let _ = fs::remove_file(&db.device_wrap);
      let _ = fs::remove_file(db.dir.join(DB_DEVICE_WRAP_TMP));
    }
  }
  let _ = fs::remove_file(db.dir.join(DB_USER_LOCK));
}

/// 把压缩包错误记进日志并收成 `Io`
///
/// # 参数
/// - `err`: 压缩包读写错误
///
/// # 返回值
/// `ArchiveError::Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let err = zip_err(error);
/// ```
fn zip_err(err: ZipError) -> ArchiveError {
  tauri_plugin_log::log::error!("archive zip: {err}");
  ArchiveError::Io
}

/// 把读写错误记进日志并收成 `Io`
///
/// # 参数
/// - `err`: 标准库读写错误
///
/// # 返回值
/// `ArchiveError::Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let err = io_err(error);
/// ```
fn io_err(err: io::Error) -> ArchiveError {
  tauri_plugin_log::log::error!("archive io: {err}");
  ArchiveError::Io
}

#[cfg(test)]
mod tests {
  use std::io::Write;
  use std::sync::Arc;

  use super::*;
  use crate::crypto::device::MemorySlot;
  use crate::db::state::DbState;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = DbState::new(dir.path().to_path_buf()).with_device(Arc::new(MemorySlot::new()));
    db.create("test-password-123", "test-password-123").expect("create");
    (dir, db)
  }

  #[test]
  fn safe_rel_rejects_parent_and_absolute() {
    assert!(safe_rel("../db/data.sqlite").is_none());
    assert!(safe_rel("/db/data.sqlite").is_none());
    assert_eq!(safe_rel("db/data.sqlite").unwrap(), PathBuf::from("db").join("data.sqlite"));
  }

  #[test]
  fn export_then_import_replaces_files_and_locks() {
    let (dir, db) = setup();
    let queue = TaskQueue::default();
    let upload = data_root(&db).join(UPLOAD_DIR).join("file");
    fs::create_dir_all(&upload).expect("upload");
    fs::write(upload.join("note.txt"), b"hello").expect("write");

    let dest = dir.path().join("archive.zip");
    let packed = export_archive(&db, &queue, &dest).expect("export");
    assert_eq!(packed.file_count, 1);
    assert!(dest.is_file());

    fs::write(upload.join("note.txt"), b"changed").expect("change");
    fs::write(upload.join("extra.txt"), b"gone").expect("extra");
    import_archive(&db, &queue, &dest).expect("import");

    assert!(!db.status().unlocked);
    assert!(!db.status().device_unlock);
    assert_eq!(fs::read(upload.join("note.txt")).expect("restored"), b"hello");
    assert!(!upload.join("extra.txt").exists());
    db.unlock("test-password-123").expect("unlock imported");
    assert!(db.status().unlocked);
  }

  #[test]
  fn import_rejects_parent_entry_without_replacing() {
    let (dir, db) = setup();
    let queue = TaskQueue::default();
    let bad = dir.path().join("bad.zip");
    let mut zip = ZipWriter::new(File::create(&bad).expect("zip"));
    zip.start_file("../outside.txt", SimpleFileOptions::default()).expect("start");
    zip.write_all(b"no").expect("body");
    zip.finish().expect("finish");

    let err = import_archive(&db, &queue, &bad).expect_err("reject");
    assert!(matches!(err, ArchiveError::Invalid));
    assert!(db.status().unlocked);
  }

  #[test]
  fn export_refuses_destination_inside_data_dir() {
    let (dir, db) = setup();
    let queue = TaskQueue::default();
    let dest = data_root(&db).join("inside.zip");
    let err = export_archive(&db, &queue, &dest).expect_err("inside");
    assert!(matches!(err, ArchiveError::PathInvalid));
    assert!(dir.path().join("data").exists());
  }
}
