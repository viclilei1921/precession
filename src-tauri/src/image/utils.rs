use std::path::{Path, PathBuf};

use crate::sidecar::{SidecarError, SidecarResult};

/// 枚举与转码共用的宽扩展名（非侧车原生格式经 PNG 桥接）
pub const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "gif", "tif", "tiff"];

/// 判断扩展名是否在可转码图片列表里
///
/// 比较前把扩展名转成小写。没有扩展名时返回 `false`。
///
/// # 参数
/// - `path`: 文件路径
///
/// # 返回值
/// 扩展名是 jpg、jpeg、png、webp、bmp、gif、tif、tiff 之一时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(is_convertible_image(Path::new("a.JPG")));
/// assert!(!is_convertible_image(Path::new("a.avif")));
/// ```
pub fn is_convertible_image(path: &Path) -> bool {
  path
    .extension()
    .and_then(|e| e.to_str())
    .map(|e| IMAGE_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
    .unwrap_or(false)
}

/// 把路径扩展名换成 `avif`
///
/// # 参数
/// - `input`: 原路径
///
/// # 返回值
/// 扩展名为 `avif` 的路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(to_avif_path(Path::new("a.png")), PathBuf::from("a.avif"));
/// ```
pub fn to_avif_path(input: &Path) -> PathBuf {
  let mut out = input.to_path_buf();
  out.set_extension("avif");
  out
}

/// 把路径扩展名换成 `jxl`
///
/// # 参数
/// - `input`: 原路径
///
/// # 返回值
/// 扩展名为 `jxl` 的路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(to_jxl_path(Path::new("a.png")), PathBuf::from("a.jxl"));
/// ```
pub fn to_jxl_path(input: &Path) -> PathBuf {
  let mut out = input.to_path_buf();
  out.set_extension("jxl");
  out
}

/// 列出目录里的可转码图片，并按路径字典序排序
///
/// # 参数
/// - `dir`: 目录
/// - `recursive`: 为真时进入子目录
///
/// # 返回值
/// 成功时返回路径字符串
///
/// # 错误
/// 不是目录时返回 `SidecarError::Invalid`。读目录失败时返回 `SidecarError::Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let files = list_image_files(dir, true)?;
/// ```
pub fn list_image_files(dir: &str, recursive: bool) -> SidecarResult<Vec<String>> {
  let root = PathBuf::from(dir);
  if !root.is_dir() {
    return Err(SidecarError::invalid(format!("不是有效目录: {dir}")));
  }

  let mut results = Vec::new();
  collect_images(&root, recursive, &mut results)?;
  results.sort();
  Ok(results.into_iter().map(|p| p.to_string_lossy().into_owned()).collect())
}

/// 把目录里的可转码图片追加到列表
///
/// 相对路径会先尝试规范化。`recursive` 为真时进入子目录。
///
/// # 参数
/// - `dir`: 目录
/// - `recursive`: 是否进入子目录
/// - `out`: 收集结果
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 读目录、读目录项或取文件类型失败时返回 `SidecarError::Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// collect_images(dir, true, &mut out)?;
/// ```
fn collect_images(dir: &Path, recursive: bool, out: &mut Vec<PathBuf>) -> SidecarResult<()> {
  let entries = std::fs::read_dir(dir).map_err(|e| SidecarError::io(format!("读取目录失败 {}: {e}", dir.display())))?;
  for entry in entries {
    let entry = entry.map_err(|e| SidecarError::io(format!("读取目录项失败: {e}")))?;
    let path = entry.path();
    let file_type = entry.file_type().map_err(|e| SidecarError::io(format!("获取文件类型失败: {e}")))?;
    if file_type.is_dir() {
      if recursive {
        collect_images(&path, true, out)?;
      }
    } else if file_type.is_file() && is_convertible_image(&path) {
      let abs = if path.is_absolute() { path } else { std::fs::canonicalize(&path).unwrap_or(path) };
      out.push(abs);
    }
  }
  Ok(())
}
