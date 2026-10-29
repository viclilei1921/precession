use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::error::FilesError;

/// 从路径收集到的文件
///
/// 目录会连同子目录一起展开，不跟随符号链接。没有符合的文件时 `files` 为空。
///
/// # 字段
/// - `files`: 符合扩展名的文件路径
/// - `scanned_directory`: 输入里至少有一个目录被展开
///
/// # 示例
/// 见 [`collect`]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Collected {
  /// 符合扩展名的文件路径
  pub files: Vec<String>,
  /// 输入里至少有一个目录被展开
  pub scanned_directory: bool,
}

/// 收集路径里符合扩展名的文件
///
/// 文件直接按扩展名判断。目录会进入子目录，跳过符号链接和读失败的子目录，并按路径排序。
/// `extensions` 为空时接受任意文件。比较时忽略大小写，扩展名可以不带点。
///
/// # 参数
/// - `paths`: 文件或目录路径
/// - `extensions`: 不带点的扩展名。空则不过滤
///
/// # 返回值
/// 成功时返回去重后的文件。没有符合的文件时 `files` 为空
///
/// # 错误
/// 路径不存在返回 `FilesError::Missing`。路径读不了返回 `FilesError::Unreadable`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let found = collect(&paths, &["jpg".to_string()])?;
/// ```
pub fn collect(paths: &[String], extensions: &[String]) -> Result<Collected, FilesError> {
  let extensions = normalize(extensions);
  let mut files = Vec::new();
  let mut scanned_directory = false;
  for path in paths {
    if path.is_empty() {
      continue;
    }
    let target = Path::new(path);
    let meta = match std::fs::metadata(target) {
      Ok(meta) => meta,
      Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Err(FilesError::Missing),
      Err(_) => return Err(FilesError::Unreadable),
    };
    if meta.is_dir() {
      scanned_directory = true;
      files.extend(list_dir(target, &extensions)?);
    } else if meta.is_file() && matches(target, &extensions) {
      files.push(path.clone());
    }
  }
  Ok(Collected { files: dedup(files), scanned_directory })
}

/// 列出目录及其子目录里符合扩展名的文件
///
/// 进入子目录。符号链接、读失败的条目和读失败的子目录会被跳过。结果按完整路径忽略大小写排序。
///
/// # 参数
/// - `dir`: 目录路径
/// - `extensions`: 已规范成带点、小写的扩展名。空则不过滤
///
/// # 返回值
/// 成功时返回目录树里的文件路径
///
/// # 错误
/// 起始目录读不了时返回 `FilesError::Unreadable`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let files = list_dir(Path::new(dir), &extensions)?;
/// ```
fn list_dir(dir: &Path, extensions: &[String]) -> Result<Vec<String>, FilesError> {
  let mut files = Vec::new();
  let mut pending = vec![dir.to_path_buf()];
  let mut root = true;
  while let Some(current) = pending.pop() {
    if let Err(err) = read_entries(&current, extensions, &mut files, &mut pending) {
      if root {
        return Err(err);
      }
    }
    root = false;
  }
  files.sort_by(|left, right| left.to_lowercase().cmp(&right.to_lowercase()));
  Ok(files)
}

/// 读取一个目录，把符合的文件和待进入的子目录分别放进两个列表
///
/// 不跟随符号链接。单个条目读失败时跳过，目录本身读失败时返回错误。
///
/// # 参数
/// - `dir`: 当前目录
/// - `extensions`: 已规范成带点、小写的扩展名。空则不过滤
/// - `files`: 已收集的文件路径
/// - `pending`: 还没进入的子目录
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 目录读不了时返回 `FilesError::Unreadable`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// read_entries(dir, &extensions, &mut files, &mut pending)?;
/// ```
fn read_entries(
  dir: &Path,
  extensions: &[String],
  files: &mut Vec<String>,
  pending: &mut Vec<PathBuf>,
) -> Result<(), FilesError> {
  let entries = std::fs::read_dir(dir).map_err(|_| FilesError::Unreadable)?;
  for entry in entries {
    let Ok(entry) = entry else {
      continue;
    };
    let Ok(kind) = entry.file_type() else {
      continue;
    };
    if kind.is_symlink() {
      continue;
    }
    let path = entry.path();
    if kind.is_dir() {
      pending.push(path);
    } else if kind.is_file() && matches(&path, extensions) {
      files.push(path.to_string_lossy().into_owned());
    }
  }
  Ok(())
}

/// 判断文件名是否符合扩展名
///
/// `extensions` 为空时任意文件名都符合。比较前把文件名转成小写。
///
/// # 参数
/// - `path`: 文件路径
/// - `extensions`: 已规范成带点、小写的扩展名
///
/// # 返回值
/// 符合时返回 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let ok = matches(Path::new("a.JPG"), &[".jpg".to_string()]);
/// ```
fn matches(path: &Path, extensions: &[String]) -> bool {
  if extensions.is_empty() {
    return true;
  }
  let Some(name) = path.file_name() else {
    return false;
  };
  let name = name.to_string_lossy().to_lowercase();
  extensions.iter().any(|ext| name.ends_with(ext))
}

/// 把扩展名规范成带点的小写形式
///
/// 去掉空白和开头的点。空字符串会被丢掉。
///
/// # 参数
/// - `extensions`: 原始扩展名
///
/// # 返回值
/// 带点的小写扩展名
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let normalized = normalize(&[" JPG ".to_string(), ".png".to_string()]);
/// ```
fn normalize(extensions: &[String]) -> Vec<String> {
  extensions
    .iter()
    .map(|ext| ext.trim().trim_start_matches('.').to_ascii_lowercase())
    .filter(|ext| !ext.is_empty())
    .map(|ext| format!(".{ext}"))
    .collect()
}

/// 按出现顺序去掉重复路径
///
/// # 参数
/// - `paths`: 待去重的路径
///
/// # 返回值
/// 保留第一次出现的路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let unique = dedup(vec!["a".to_string(), "a".to_string(), "b".to_string()]);
/// ```
fn dedup(paths: Vec<String>) -> Vec<String> {
  let mut seen = HashSet::new();
  let mut unique = Vec::new();
  for path in paths {
    if seen.insert(path.clone()) {
      unique.push(path);
    }
  }
  unique
}

#[cfg(test)]
mod tests {
  use super::*;

  fn names(paths: &[String]) -> Vec<String> {
    paths
      .iter()
      .map(|path| Path::new(path).file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default())
      .collect()
  }

  #[test]
  fn lists_matching_files_in_nested_directories() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("b.PNG"), b"b").expect("write");
    std::fs::write(dir.path().join("a.jpg"), b"a").expect("write");
    std::fs::write(dir.path().join("note.txt"), b"t").expect("write");
    let nested = dir.path().join("nested");
    std::fs::create_dir(&nested).expect("mkdir");
    std::fs::write(nested.join("c.jpg"), b"c").expect("write");
    let deep = nested.join("deep");
    std::fs::create_dir(&deep).expect("mkdir");
    std::fs::write(deep.join("d.jpg"), b"d").expect("write");

    let found = collect(
      &[dir.path().to_string_lossy().into_owned()],
      &["jpg".to_string(), "png".to_string()],
    )
    .expect("collect");
    assert!(found.scanned_directory);
    assert_eq!(
      names(&found.files),
      vec!["a.jpg".to_string(), "b.PNG".to_string(), "c.jpg".to_string(), "d.jpg".to_string()]
    );
  }

  #[test]
  fn empty_extensions_keep_every_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("a.jpg"), b"a").expect("write");
    std::fs::write(dir.path().join("note.txt"), b"t").expect("write");
    let nested = dir.path().join("nested");
    std::fs::create_dir(&nested).expect("mkdir");
    std::fs::write(nested.join("c.jpg"), b"c").expect("write");

    let found = collect(&[dir.path().to_string_lossy().into_owned()], &[]).expect("collect");
    assert!(found.scanned_directory);
    assert_eq!(
      names(&found.files),
      vec!["a.jpg".to_string(), "c.jpg".to_string(), "note.txt".to_string()]
    );
  }

  #[test]
  fn keeps_a_matching_file_and_drops_others() {
    let dir = tempfile::tempdir().expect("tempdir");
    let image = dir.path().join("a.jpg");
    let note = dir.path().join("note.txt");
    std::fs::write(&image, b"a").expect("write");
    std::fs::write(&note, b"t").expect("write");

    let found = collect(
      &[image.to_string_lossy().into_owned(), note.to_string_lossy().into_owned()],
      &["jpg".to_string()],
    )
    .expect("collect");
    assert!(!found.scanned_directory);
    assert_eq!(names(&found.files), vec!["a.jpg".to_string()]);
  }

  #[test]
  fn missing_path_is_an_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("missing.jpg");
    let err = collect(&[missing.to_string_lossy().into_owned()], &["jpg".to_string()]).expect_err("missing");
    assert!(matches!(err, FilesError::Missing));
  }
}
