use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use super::error::ViewerError;
use super::folder::is_image;

/// 可以交给页面显示的文件
///
/// JPEG XL 的 `display_path` 是缓存 PNG。其他格式就是原文件。
///
/// # 字段
/// - `display_path`: `<img>` 要读的路径
/// - `width`: 像素宽
/// - `height`: 像素高
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewerDisplay {
  /// `<img>` 要读的路径
  pub display_path: String,
  /// 像素宽
  pub width: u32,
  /// 像素高
  pub height: u32,
}

/// 准备一张可以显示的图
///
/// JPEG XL 解出第一帧，按路径、修改时间和大小写进缓存。缓存还在时不再解码。其他格式直接返回原路径；宽高读得到就带上，读不到时为 0。
///
/// # 参数
/// - `app`: 用来找缓存目录
/// - `path`: 图片路径
///
/// # 返回值
/// 成功时返回显示路径和像素尺寸
///
/// # 错误
/// 不是图片返回 `ViewerError::Unsupported`。文件读不了返回 `ViewerError::Unreadable`。JPEG XL 解码或写缓存失败返回 `ViewerError::Decode`
///
/// # Panics
/// 不会主动 panic
pub fn prepare(app: &AppHandle, path: &str) -> Result<ViewerDisplay, ViewerError> {
  let file = PathBuf::from(path);
  if !file.is_file() {
    return Err(if file.exists() { ViewerError::Unsupported } else { ViewerError::Missing });
  }
  if !is_image(&file) {
    return Err(ViewerError::Unsupported);
  }
  let ext = file.extension().map(|ext| ext.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
  if ext == "jxl" {
    return prepare_jxl(app, &file);
  }
  let (width, height) = header_size(&file).unwrap_or((0, 0));
  Ok(ViewerDisplay { display_path: file.to_string_lossy().into_owned(), width, height })
}

/// 把 JPEG XL 解成缓存 PNG
fn prepare_jxl(app: &AppHandle, file: &Path) -> Result<ViewerDisplay, ViewerError> {
  let meta = std::fs::metadata(file).map_err(|_| ViewerError::Unreadable)?;
  let modified = meta
    .modified()
    .ok()
    .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
    .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
    .unwrap_or(0);
  let cache_dir = app.path().app_cache_dir().map_err(|_| ViewerError::Decode)?.join("viewer");
  std::fs::create_dir_all(&cache_dir).map_err(|_| ViewerError::Decode)?;
  let key = path_key(&file.to_string_lossy());
  let cache = cache_dir.join(format!("{key}-{modified}-{}.png", meta.len()));
  if cache.is_file() {
    if let Some((width, height)) = header_size(&cache) {
      let _ = app.asset_protocol_scope().allow_file(&cache);
      return Ok(ViewerDisplay { display_path: cache.to_string_lossy().into_owned(), width, height });
    }
  }
  clear_old(&cache_dir, &key, &cache);
  let (width, height) = decode_jxl(file, &cache)?;
  let _ = app.asset_protocol_scope().allow_file(&cache);
  Ok(ViewerDisplay { display_path: cache.to_string_lossy().into_owned(), width, height })
}

/// 解出第一帧并写成 PNG
fn decode_jxl(file: &Path, cache: &Path) -> Result<(u32, u32), ViewerError> {
  let image = jxl_oxide::JxlImage::builder().open(file).map_err(|_| ViewerError::Decode)?;
  let render = image.render_frame(0).map_err(|_| ViewerError::Decode)?;
  let frame = render.image_all_channels();
  let (width, height, pixels) = rgba(&frame)?;
  let image = image::RgbaImage::from_raw(width, height, pixels).ok_or(ViewerError::Decode)?;
  image.save(cache).map_err(|_| ViewerError::Decode)?;
  Ok((width, height))
}

/// 把解码缓冲收成 RGBA 字节。采样按 0 到 1 收到 0 到 255
fn rgba(frame: &jxl_oxide::FrameBuffer) -> Result<(u32, u32, Vec<u8>), ViewerError> {
  let width = u32::try_from(frame.width()).map_err(|_| ViewerError::Decode)?;
  let height = u32::try_from(frame.height()).map_err(|_| ViewerError::Decode)?;
  let channels = frame.channels();
  if channels == 0 || width == 0 || height == 0 {
    return Err(ViewerError::Decode);
  }
  let count = (width as usize).checked_mul(height as usize).ok_or(ViewerError::Decode)?;
  let buf = frame.buf();
  let mut out = Vec::with_capacity(count.saturating_mul(4));
  for pixel in 0..count {
    let base = pixel * channels;
    let sample = |offset: usize| -> u8 {
      let value = buf.get(base + offset).copied().unwrap_or(0.0);
      (value.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    let (red, green, blue, alpha) = match channels {
      1 => {
        let gray = sample(0);
        (gray, gray, gray, 255)
      }
      2 => {
        let gray = sample(0);
        (gray, gray, gray, sample(1))
      }
      _ => (sample(0), sample(1), sample(2), if channels >= 4 { sample(3) } else { 255 }),
    };
    out.extend_from_slice(&[red, green, blue, alpha]);
  }
  Ok((width, height, out))
}

/// 路径的稳定短名，用来把同一张图的缓存放在一起
fn path_key(path: &str) -> String {
  let mut hasher = DefaultHasher::new();
  path.hash(&mut hasher);
  format!("{:016x}", hasher.finish())
}

/// 删掉同一张图的旧缓存
fn clear_old(dir: &Path, key: &str, keep: &Path) {
  let Ok(entries) = std::fs::read_dir(dir) else {
    return;
  };
  let prefix = format!("{key}-");
  for entry in entries.flatten() {
    let path = entry.path();
    if path == keep {
      continue;
    }
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
      continue;
    };
    if name.starts_with(&prefix) {
      let _ = std::fs::remove_file(path);
    }
  }
}

/// 只读文件头里的宽高
fn header_size(path: &Path) -> Option<(u32, u32)> {
  let reader = image::ImageReader::open(path).ok()?.with_guessed_format().ok()?;
  reader.into_dimensions().ok()
}
