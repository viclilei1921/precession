//! 上传文件的落盘位置。和 `data/db` 平级，放在 `data/upload/{kind}/{id}.{ext}`。
//!
//! 数据库只存相对数据根目录的路径，例如 `upload/image/{id}.jpg`。导入先写到 `upload/.tmp`，
//! 提交时在 [`commit_guard`] 下改名到正式位置，再插入文件记录。孤儿清理也拿同一把锁，
//! 所以不会删到正在提交的文件。

use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use super::constants::{EXT_MAX, SIDE_SUFFIXES, TMP_DIR, UPLOAD_DIR};
use super::error::UploadError;
use super::repository;
use crate::db::state::DbState;

static COMMIT: Mutex<()> = Mutex::new(());

/// 拿到提交和清理共用的锁
///
/// 锁中毒时照样取回守卫，里面没有要保护的数据。
///
/// # 返回值
/// 持有期间其他提交和清理会等待
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let _guard = commit_guard();
/// ```
pub(crate) fn commit_guard() -> MutexGuard<'static, ()> {
  COMMIT.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 取数据库目录的父目录，没有父目录时用数据库目录本身
///
/// # 参数
/// - `db`: 数据库状态
///
/// # 返回值
/// 数据根目录，即 `app_local_data_dir/data`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let root = data_root(db);
/// ```
pub(crate) fn data_root(db: &DbState) -> PathBuf {
  db.dir.parent().map(Path::to_path_buf).unwrap_or_else(|| db.dir.clone())
}

/// 把相对路径接到数据根目录上
///
/// `rel` 已经是绝对路径时原样返回。
///
/// # 参数
/// - `db`: 数据库状态
/// - `rel`: 相对或绝对路径
///
/// # 返回值
/// 解析后的路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let path = resolve(db, "upload/image/a.jpg");
/// ```
pub(crate) fn resolve(db: &DbState, rel: &str) -> PathBuf {
  let path = PathBuf::from(rel);
  if path.is_absolute() { path } else { data_root(db).join(path) }
}

/// 返回导入临时目录下的文件路径
///
/// # 参数
/// - `db`: 数据库状态
/// - `name`: 文件名
///
/// # 返回值
/// `data/upload/.tmp/{name}`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let staged = tmp_path(db, "id.part");
/// ```
pub(crate) fn tmp_path(db: &DbState, name: &str) -> PathBuf {
  data_root(db).join(UPLOAD_DIR).join(TMP_DIR).join(name)
}

/// 取路径的扩展名，只保留 ASCII 字母和数字
///
/// 转成小写。为空、超过 10 个字符或含其他字符时返回 `None`。
///
/// # 参数
/// - `path`: 文件路径
///
/// # 返回值
/// 小写扩展名
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(ext_of(Path::new("a/B.JPG")).as_deref(), Some("jpg"));
/// assert_eq!(ext_of(Path::new("a/b")), None);
/// ```
pub(crate) fn ext_of(path: &Path) -> Option<String> {
  let ext = path.extension()?.to_str()?.to_ascii_lowercase();
  let valid = !ext.is_empty() && ext.len() <= EXT_MAX && ext.chars().all(|c| c.is_ascii_alphanumeric());
  valid.then_some(ext)
}

/// 拼出上传文件的相对路径
///
/// # 参数
/// - `kind`: 文件类型，用作子目录
/// - `id`: 文件 ID
/// - `ext`: 小写扩展名，可以没有
///
/// # 返回值
/// `upload/{kind}/{id}.{ext}`，没有扩展名时是 `upload/{kind}/{id}`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(rel_path("image", "a", Some("jpg")), "upload/image/a.jpg");
/// assert_eq!(rel_path("file", "a", None), "upload/file/a");
/// ```
pub(crate) fn rel_path(kind: &str, id: &str, ext: Option<&str>) -> String {
  match ext {
    Some(ext) => format!("{UPLOAD_DIR}/{kind}/{id}.{ext}"),
    None => format!("{UPLOAD_DIR}/{kind}/{id}"),
  }
}

/// 删除上传目录里的一个文件
///
/// 只处理 `upload/` 开头、不含 `..` 的相对路径，其余路径一律不删，避免删到用户自己的文件。删除失败时忽略。
///
/// # 参数
/// - `db`: 数据库状态
/// - `rel`: 文件记录里的相对路径
///
/// # 返回值
/// 不返回数据
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// remove(db, "upload/image/a.jpg");
/// ```
pub(crate) fn remove(db: &DbState, rel: &str) {
  if !is_upload_rel(rel) {
    return;
  }
  let _ = fs::remove_file(resolve(db, rel));
}

/// 判断路径是否是上传目录里的相对路径
///
/// # 参数
/// - `rel`: 相对路径
///
/// # 返回值
/// 以 `upload/` 开头且全是普通路径段时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(is_upload_rel("upload/image/a.jpg"));
/// assert!(!is_upload_rel("upload/../db/data.sqlite"));
/// ```
fn is_upload_rel(rel: &str) -> bool {
  let path = Path::new(rel);
  let mut parts = path.components();
  let head_ok = matches!(parts.next(), Some(Component::Normal(head)) if head == UPLOAD_DIR);
  head_ok && parts.all(|part| matches!(part, Component::Normal(_)))
}

/// 清空导入临时目录
///
/// 只在启动、任务工人开始之前调用，此时不会有导入在写临时文件。删除失败时忽略。
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
/// clear_tmp(db);
/// ```
pub(crate) fn clear_tmp(db: &DbState) {
  let _ = fs::remove_dir_all(data_root(db).join(UPLOAD_DIR).join(TMP_DIR));
}

/// 在提交锁下删掉上传目录里没有文件记录的文件
///
/// # 参数
/// - `db`: 数据库状态
///
/// # 返回值
/// 删掉的文件数
///
/// # 错误
/// 数据库未解锁返回 `UploadError::Locked`，查询失败返回 `UploadError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let removed = sweep_now(db)?;
/// ```
pub(crate) fn sweep_now(db: &DbState) -> Result<usize, UploadError> {
  let _guard = commit_guard();
  let keep = db.with_conn(repository::all_paths)?;
  Ok(sweep(&data_root(db).join(UPLOAD_DIR), &keep))
}

/// 在后台线程里执行一次 [`sweep_now`]
///
/// 失败只写日志。
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
/// sweep_later(db);
/// ```
pub(crate) fn sweep_later(db: &DbState) {
  let db = db.clone();
  std::thread::spawn(move || match sweep_now(&db) {
    Ok(0) => {}
    Ok(n) => tauri_plugin_log::log::info!("upload sweep removed {n} files"),
    Err(err) => tauri_plugin_log::log::warn!("upload sweep: {err}"),
  });
}

/// 数据库打开后的钩子：清理孤儿文件
///
/// # 参数
/// - `db`: 刚打开的数据库状态
///
/// # 返回值
/// 不返回数据
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// on_open(db);
/// ```
pub(crate) fn on_open(db: &DbState) {
  sweep_later(db);
}

/// 删掉上传目录各子目录里不在 `keep` 中的文件
///
/// 跳过临时目录。文件名是某个保留文件加上 `.part`、`.bak`、`.aside` 时也跳过，那是加解密过程中的旁路文件。目录读不了时跳过。
///
/// # 参数
/// - `upload`: 上传目录
/// - `keep`: 要保留的相对路径，形如 `upload/image/a.jpg`
///
/// # 返回值
/// 删掉的文件数
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let removed = sweep(upload, &keep);
/// ```
fn sweep(upload: &Path, keep: &HashSet<String>) -> usize {
  let Ok(dirs) = fs::read_dir(upload) else {
    return 0;
  };
  let mut removed = 0;
  for dir in dirs.flatten() {
    let dir_name = dir.file_name().to_string_lossy().into_owned();
    if dir_name == TMP_DIR || !dir.path().is_dir() {
      continue;
    }
    let Ok(files) = fs::read_dir(dir.path()) else {
      continue;
    };
    for file in files.flatten() {
      if !file.path().is_file() {
        continue;
      }
      let rel = format!("{UPLOAD_DIR}/{dir_name}/{}", file.file_name().to_string_lossy());
      if keep.contains(&rel) || is_side_of_kept(&rel, keep) {
        continue;
      }
      if fs::remove_file(file.path()).is_ok() {
        removed += 1;
      }
    }
  }
  removed
}

/// 判断文件是不是某个保留文件的旁路文件
///
/// # 参数
/// - `rel`: 候选相对路径
/// - `keep`: 要保留的相对路径
///
/// # 返回值
/// 去掉 `.part`、`.bak` 或 `.aside` 后在 `keep` 里时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let side = is_side_of_kept("upload/image/a.jpg.part", &keep);
/// ```
fn is_side_of_kept(rel: &str, keep: &HashSet<String>) -> bool {
  SIDE_SUFFIXES.iter().any(|suffix| {
    rel.strip_suffix(suffix).and_then(|rest| rest.strip_suffix('.')).is_some_and(|base| keep.contains(base))
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rel_path_and_ext() {
    assert_eq!(ext_of(Path::new("a/B.JPG")).as_deref(), Some("jpg"));
    assert_eq!(ext_of(Path::new("a/b")), None);
    assert_eq!(ext_of(Path::new("a/b.tar gz")), None);
    assert_eq!(ext_of(Path::new("a/b.verylongextension")), None);
    assert_eq!(rel_path("image", "x", Some("png")), "upload/image/x.png");
    assert_eq!(rel_path("file", "x", None), "upload/file/x");
  }

  #[test]
  fn only_upload_paths_are_removable() {
    assert!(is_upload_rel("upload/image/a.jpg"));
    assert!(!is_upload_rel("media/a"));
    assert!(!is_upload_rel("upload/../db/data.sqlite"));
    assert!(!is_upload_rel("/abs/upload/a"));
  }

  #[test]
  fn sweep_keeps_known_side_and_tmp_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let upload = dir.path().join(UPLOAD_DIR);
    let image = upload.join("image");
    let tmp = upload.join(TMP_DIR);
    fs::create_dir_all(&image).expect("image dir");
    fs::create_dir_all(&tmp).expect("tmp dir");
    for name in ["kept.jpg", "kept.jpg.part", "kept.jpg.bak", "orphan.jpg", "orphan.jpg.part"] {
      fs::write(image.join(name), b"x").expect("write");
    }
    fs::write(tmp.join("staged.part"), b"x").expect("tmp");

    let keep: HashSet<String> = ["upload/image/kept.jpg".to_string()].into_iter().collect();
    assert_eq!(sweep(&upload, &keep), 2);
    assert!(image.join("kept.jpg").exists());
    assert!(image.join("kept.jpg.part").exists());
    assert!(image.join("kept.jpg.bak").exists());
    assert!(!image.join("orphan.jpg").exists());
    assert!(!image.join("orphan.jpg.part").exists());
    assert!(tmp.join("staged.part").exists());
  }
}
