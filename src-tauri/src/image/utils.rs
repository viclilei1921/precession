use std::path::{Path, PathBuf};

use crate::sidecar::{SidecarError, SidecarResult};

/// 枚举与转码共用的宽扩展名（非侧车原生格式经 PNG 桥接）
pub const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "gif", "tif", "tiff"];

pub fn is_convertible_image(path: &Path) -> bool {
  path
    .extension()
    .and_then(|e| e.to_str())
    .map(|e| IMAGE_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
    .unwrap_or(false)
}

/// 将路径扩展名替换为 `.avif`
pub fn to_avif_path(input: &Path) -> PathBuf {
  let mut out = input.to_path_buf();
  out.set_extension("avif");
  out
}

/// 将路径扩展名替换为 `.jxl`
pub fn to_jxl_path(input: &Path) -> PathBuf {
  let mut out = input.to_path_buf();
  out.set_extension("jxl");
  out
}

/// 枚举目录下的可转码图片路径（绝对路径，按字典序）
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
