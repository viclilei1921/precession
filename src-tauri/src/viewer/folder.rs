use std::path::{Path, PathBuf};

use serde::Serialize;

use super::error::ViewerError;

/// 查看器直接显示，或解码后显示的扩展名。不带点，比较时忽略大小写。
pub const EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp", "avif", "jxl"];

/// 同一层里的一张图片
///
/// JPEG XL 的宽高要等解码后才有，这里是空。AVIF 读不到文件头时也是空，页面再用图片的原始像素补上。
///
/// # 字段
/// - `path`: 文件路径
/// - `name`: 文件名
/// - `extension`: 小写扩展名，不带点
/// - `bytes`: 字节数
/// - `modified_ms`: 修改时间，Unix 毫秒。读不到时为 0
/// - `width`: 像素宽
/// - `height`: 像素高
///
/// # 示例
/// 见 [`read`]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewerFile {
  /// 文件路径
  pub path: String,
  /// 文件名
  pub name: String,
  /// 小写扩展名，不带点
  pub extension: String,
  /// 字节数
  pub bytes: u64,
  /// 修改时间，Unix 毫秒
  pub modified_ms: u64,
  /// 像素宽。还不知道时为空
  pub width: Option<u32>,
  /// 像素高。还不知道时为空
  pub height: Option<u32>,
}

/// 一个文件夹里可查看的图片
///
/// 只列这一层，不进入子目录。`files` 按文件名忽略大小写排序。
///
/// # 字段
/// - `directory`: 文件夹路径
/// - `files`: 这一层的图片。没有时为空
/// - `index`: `files` 里当前图片的下标。空文件夹时为 0
///
/// # 示例
/// 见 [`read`]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewerFolder {
  /// 文件夹路径
  pub directory: String,
  /// 这一层的图片
  pub files: Vec<ViewerFile>,
  /// 当前图片下标
  pub index: usize,
}

/// 列出图片所在文件夹这一层的其他图片
///
/// 传入文件时，下标落在这张图上。传入文件夹时，下标为 0。空文件夹返回空列表，不算错误。
///
/// # 参数
/// - `path`: 图片或文件夹路径
///
/// # 返回值
/// 成功时返回这一层的图片和当前下标
///
/// # 错误
/// 路径不存在返回 `ViewerError::Missing`。文件不是支持的图片返回 `ViewerError::Unsupported`。目录读不了返回 `ViewerError::Unreadable`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let folder = read(r"D:\photos\a.png")?;
/// ```
pub fn read(path: &str) -> Result<ViewerFolder, ViewerError> {
  if path.trim().is_empty() {
    return Err(ViewerError::Missing);
  }
  let target = PathBuf::from(path);
  let meta = std::fs::metadata(&target).map_err(map_io)?;
  let directory = if meta.is_dir() {
    target.clone()
  } else if meta.is_file() {
    if !is_image(&target) {
      return Err(ViewerError::Unsupported);
    }
    target.parent().map(Path::to_path_buf).filter(|dir| !dir.as_os_str().is_empty()).ok_or(ViewerError::Unreadable)?
  } else {
    return Err(ViewerError::Unsupported);
  };

  let mut files = Vec::new();
  for entry in std::fs::read_dir(&directory).map_err(|_| ViewerError::Unreadable)?.flatten() {
    let file_path = entry.path();
    let file_meta = match std::fs::metadata(&file_path) {
      Ok(meta) if meta.is_file() => meta,
      _ => continue,
    };
    if !is_image(&file_path) {
      continue;
    }
    files.push(describe(&file_path, &file_meta));
  }
  files.sort_by(|left, right| {
    left.name.to_ascii_lowercase().cmp(&right.name.to_ascii_lowercase()).then_with(|| left.name.cmp(&right.name))
  });

  let index = if meta.is_dir() {
    0
  } else {
    files.iter().position(|file| same_file(&file.path, &target)).ok_or(ViewerError::Unsupported)?
  };

  Ok(ViewerFolder { directory: directory.to_string_lossy().into_owned(), files, index })
}

/// 扩展名是查看器接受的图片
///
/// # 参数
/// - `path`: 文件路径
///
/// # 返回值
/// 扩展名在 [`EXTENSIONS`] 里时为真
///
/// # Panics
/// 不会主动 panic
pub fn is_image(path: &Path) -> bool {
  let ext = extension(path);
  EXTENSIONS.iter().any(|item| *item == ext)
}

/// 小写扩展名，不带点。没有扩展名时为空
fn extension(path: &Path) -> String {
  path.extension().map(|ext| ext.to_string_lossy().to_ascii_lowercase()).unwrap_or_default()
}

/// 把一个文件收成列表项
fn describe(path: &Path, meta: &std::fs::Metadata) -> ViewerFile {
  let extension = extension(path);
  let size = if extension == "jxl" || extension == "avif" { None } else { header_size(path) };
  ViewerFile {
    path: path.to_string_lossy().into_owned(),
    name: path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default(),
    extension,
    bytes: meta.len(),
    modified_ms: modified_ms(meta),
    width: size.map(|(width, _)| width),
    height: size.map(|(_, height)| height),
  }
}

/// 只读文件头里的宽高。读不到时为空
fn header_size(path: &Path) -> Option<(u32, u32)> {
  let reader = image::ImageReader::open(path).ok()?.with_guessed_format().ok()?;
  reader.into_dimensions().ok()
}

/// 修改时间的 Unix 毫秒。读不到时为 0
fn modified_ms(meta: &std::fs::Metadata) -> u64 {
  meta
    .modified()
    .ok()
    .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
    .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
    .unwrap_or(0)
}

/// 同一目录下按文件名忽略大小写比较
fn same_file(listed: &str, requested: &Path) -> bool {
  let Some(listed_name) = Path::new(listed).file_name() else {
    return false;
  };
  let Some(requested_name) = requested.file_name() else {
    return false;
  };
  listed_name.to_string_lossy().eq_ignore_ascii_case(&requested_name.to_string_lossy())
}

/// 把一张图片改成新的文件名
///
/// 只改同一层里的名字，不移动目录，也不改扩展名。只改大小写时会先挪到临时名字，避免大小写不敏感的磁盘拒绝覆盖自己。
///
/// # 参数
/// - `path`: 现在的图片路径
/// - `name`: 新文件名，不含目录。扩展名必须和原来相同
///
/// # 返回值
/// 成功时返回改名后的文件
///
/// # 错误
/// 路径不存在返回 `ViewerError::Missing`。不是支持的图片返回 `ViewerError::Unsupported`。名字不合法返回 `ViewerError::InvalidName`。扩展名变了返回 `ViewerError::Extension`。目标已经存在返回 `ViewerError::Exists`。系统拒绝改名返回 `ViewerError::Rename`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let file = rename(r"D:\photos\a.jpg", "b.jpg")?;
/// ```
pub fn rename(path: &str, name: &str) -> Result<ViewerFile, ViewerError> {
  if path.trim().is_empty() {
    return Err(ViewerError::Missing);
  }
  let source = PathBuf::from(path);
  let meta = std::fs::metadata(&source).map_err(map_io)?;
  if !meta.is_file() || !is_image(&source) {
    return Err(ViewerError::Unsupported);
  }
  let file_name = checked_file_name(name)?;
  let parent = source.parent().filter(|dir| !dir.as_os_str().is_empty()).ok_or(ViewerError::Unreadable)?;
  let target = parent.join(&file_name);
  if extension(&target) != extension(&source) || !is_image(&target) {
    return Err(ViewerError::Extension);
  }
  let current_name = source.file_name().map(|item| item.to_string_lossy().into_owned()).unwrap_or_default();
  if current_name == file_name {
    return Ok(describe(&source, &meta));
  }
  if target.exists() {
    if current_name.to_lowercase() == file_name.to_lowercase() {
      rename_case(&source, &target)?;
    } else {
      return Err(ViewerError::Exists);
    }
  } else {
    std::fs::rename(&source, &target).map_err(map_rename)?;
  }
  let renamed = std::fs::metadata(&target).unwrap_or(meta);
  Ok(describe(&target, &renamed))
}

/// 收下可以作为文件名的字符串。空着、带路径，或含有系统不接受的字符时失败
fn checked_file_name(name: &str) -> Result<String, ViewerError> {
  let name = name.trim();
  if name.is_empty()
    || name == "."
    || name == ".."
    || name.len() > 255
    || name.ends_with([' ', '.'])
    || name.chars().any(|ch| ch.is_control() || matches!(ch, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
  {
    return Err(ViewerError::InvalidName);
  }
  let stem = name.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(name);
  if stem.is_empty() || reserved_windows_name(stem) {
    return Err(ViewerError::InvalidName);
  }
  let path = Path::new(name);
  if path.file_name().is_none_or(|item| item != name) {
    return Err(ViewerError::InvalidName);
  }
  Ok(name.to_owned())
}

/// Windows 设备名。带上扩展名也不允许，例如 `CON.jpg`
fn reserved_windows_name(stem: &str) -> bool {
  let head = stem.split('.').next().unwrap_or(stem);
  matches!(
    head.to_ascii_lowercase().as_str(),
    "con"
      | "prn"
      | "aux"
      | "nul"
      | "com1"
      | "com2"
      | "com3"
      | "com4"
      | "com5"
      | "com6"
      | "com7"
      | "com8"
      | "com9"
      | "lpt1"
      | "lpt2"
      | "lpt3"
      | "lpt4"
      | "lpt5"
      | "lpt6"
      | "lpt7"
      | "lpt8"
      | "lpt9"
  )
}

/// 只改大小写。先挪开再改回，这样大小写不敏感的磁盘不会说目标已经存在
fn rename_case(source: &Path, target: &Path) -> Result<(), ViewerError> {
  let parent = source.parent().filter(|dir| !dir.as_os_str().is_empty()).ok_or(ViewerError::Unreadable)?;
  let temp = temporary_path(parent);
  std::fs::rename(source, &temp).map_err(map_rename)?;
  if let Err(err) = std::fs::rename(&temp, target) {
    let _ = std::fs::rename(&temp, source);
    return Err(map_rename(err));
  }
  Ok(())
}

/// 同一层里一个不会被当成图片的临时名字
fn temporary_path(parent: &Path) -> PathBuf {
  let token =
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|duration| duration.as_nanos()).unwrap_or(0);
  let mut path = parent.join(format!(".{token}.part"));
  let mut salt = 0u32;
  while path.exists() && salt < 100 {
    salt += 1;
    path = parent.join(format!(".{token}-{salt}.part"));
  }
  path
}

/// 把文件系统错误收成看图错误
fn map_io(err: std::io::Error) -> ViewerError {
  if err.kind() == std::io::ErrorKind::NotFound { ViewerError::Missing } else { ViewerError::Unreadable }
}

/// 改名失败时区分「已经存在」和其余拒绝
fn map_rename(err: std::io::Error) -> ViewerError {
  match err.kind() {
    std::io::ErrorKind::NotFound => ViewerError::Missing,
    std::io::ErrorKind::AlreadyExists => ViewerError::Exists,
    _ => ViewerError::Rename,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn lists_only_the_current_directory() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("b.PNG"), b"b").expect("write");
    std::fs::write(dir.path().join("a.jpg"), b"a").expect("write");
    std::fs::write(dir.path().join("note.txt"), b"t").expect("write");
    std::fs::write(dir.path().join("c.jxl"), b"j").expect("write");
    let nested = dir.path().join("nested");
    std::fs::create_dir(&nested).expect("mkdir");
    std::fs::write(nested.join("d.jpg"), b"d").expect("write");

    let folder = read(&dir.path().join("a.jpg").to_string_lossy()).expect("read");
    let names: Vec<_> = folder.files.iter().map(|file| file.name.as_str()).collect();
    assert_eq!(names, vec!["a.jpg", "b.PNG", "c.jxl"]);
    assert_eq!(folder.index, 0);
    assert!(folder.files.iter().any(|file| file.extension == "png"));
    assert!(folder.files.iter().find(|file| file.extension == "jxl").is_some_and(|file| file.width.is_none()));
  }

  #[test]
  fn empty_directory_is_not_an_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let folder = read(&dir.path().to_string_lossy()).expect("read");
    assert!(folder.files.is_empty());
    assert_eq!(folder.index, 0);
  }

  #[test]
  fn rejects_other_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let note = dir.path().join("note.tif");
    std::fs::write(&note, b"t").expect("write");
    let err = read(&note.to_string_lossy()).expect_err("tif");
    assert!(matches!(err, ViewerError::Unsupported));
  }

  #[test]
  fn renames_within_the_same_directory() {
    let dir = tempfile::tempdir().expect("tempdir");
    let from = dir.path().join("a.jpg");
    std::fs::write(&from, b"a").expect("write");
    std::fs::write(dir.path().join("taken.jpg"), b"t").expect("write");

    let renamed = rename(&from.to_string_lossy(), "  shore.jpg  ").expect("rename");
    assert_eq!(renamed.name, "shore.jpg");
    assert!(dir.path().join("shore.jpg").is_file());
    assert!(!from.exists());

    let same = rename(&renamed.path, "shore.jpg").expect("same");
    assert_eq!(same.path, renamed.path);

    let cased = rename(&renamed.path, "Shore.jpg").expect("case");
    assert_eq!(cased.name, "Shore.jpg");
    let listed = std::fs::read_dir(dir.path()).expect("dir").flatten().any(|entry| entry.file_name() == "Shore.jpg");
    assert!(listed);

    let exists = rename(&cased.path, "taken.jpg").expect_err("exists");
    assert!(matches!(exists, ViewerError::Exists));
    let extension = rename(&cased.path, "Shore.png").expect_err("extension");
    assert!(matches!(extension, ViewerError::Extension));
    let invalid = rename(&cased.path, "a/b.jpg").expect_err("slash");
    assert!(matches!(invalid, ViewerError::InvalidName));
    let blank = rename(&cased.path, "   ").expect_err("blank");
    assert!(matches!(blank, ViewerError::InvalidName));
    let missing = rename(&dir.path().join("gone.jpg").to_string_lossy(), "c.jpg").expect_err("missing");
    assert!(matches!(missing, ViewerError::Missing));
  }
}
