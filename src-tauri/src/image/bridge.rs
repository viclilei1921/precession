//! 非侧车原生格式：解码为临时 PNG，并尽力提取 EXIF 供注入。

use std::fs;
use std::path::{Path, PathBuf};

use tauri::AppHandle;

use crate::image::utils::IMAGE_EXTENSIONS;
use crate::sidecar::cache::get_cache_temp_dir;
use crate::sidecar::{SidecarError, SidecarResult};
use crate::utils::id::new_uuid_v4;

/// avifenc 原生可读
pub const AVIF_NATIVE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png"];
/// cjxl 原生可读
pub const JXL_NATIVE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif"];

/// 编码目标
///
/// AVIF 原生可读 jpg、jpeg、png。JXL 再加 gif。
///
/// # 变体
/// - `Avif`: 交给 avifenc。原生可读 jpg、jpeg、png
/// - `Jxl`: 交给 cjxl。原生可读 jpg、jpeg、png、gif
///
/// # 示例
/// 见 [`is_native_for`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncodeTarget {
  /// 交给 avifenc。原生可读 jpg、jpeg、png
  Avif,
  /// 交给 cjxl。原生可读 jpg、jpeg、png、gif
  Jxl,
}

/// 桥接产物：像素走 PNG；可选 EXIF 原始载荷文件。
///
/// 丢弃时删掉这两个临时文件。删除失败时忽略。
///
/// # 字段
/// - `png_path`: 临时 PNG
/// - `exif_path`: EXIF 原始载荷。没有提出载荷时为空
///
/// # 示例
/// 见 [`prepare_bridge_input`]。
pub struct BridgeInput {
  /// 临时 PNG
  pub png_path: PathBuf,
  /// EXIF 原始载荷。没有提出载荷时为空
  pub exif_path: Option<PathBuf>,
}

impl BridgeInput {
  /// 删掉临时 PNG 和 EXIF 文件
  ///
  /// 删除失败时忽略。
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// bridge.cleanup();
  /// ```
  pub fn cleanup(&self) {
    let _ = fs::remove_file(&self.png_path);
    if let Some(ref p) = self.exif_path {
      let _ = fs::remove_file(p);
    }
  }
}

impl Drop for BridgeInput {
  /// 丢弃时删掉临时文件
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// drop(bridge);
  /// ```
  fn drop(&mut self) {
    self.cleanup();
  }
}

/// 取出路径扩展名并转成小写
///
/// # 参数
/// - `path`: 文件路径
///
/// # 返回值
/// 有扩展名时返回小写文本，否则 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(path_ext_lower("a.JPG"), Some("jpg".to_string()));
/// assert_eq!(path_ext_lower("a"), None);
/// ```
pub fn path_ext_lower(path: &str) -> Option<String> {
  Path::new(path).extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase())
}

/// 判断扩展名是否在可转码列表里
///
/// # 参数
/// - `ext`: 小写扩展名
///
/// # 返回值
/// 在 jpg、jpeg、png、webp、bmp、gif、tif、tiff 中时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(is_supported_image_ext("png"));
/// assert!(!is_supported_image_ext("PNG"));
/// ```
pub fn is_supported_image_ext(ext: &str) -> bool {
  IMAGE_EXTENSIONS.iter().any(|ok| *ok == ext)
}

/// 判断扩展名是否能被该编码器直接读取
///
/// AVIF 接受 jpg、jpeg、png。JXL 再加 gif。
///
/// # 参数
/// - `ext`: 小写扩展名
/// - `target`: 编码目标
///
/// # 返回值
/// 可以直接读取时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(is_native_for("gif", EncodeTarget::Jxl));
/// assert!(!is_native_for("gif", EncodeTarget::Avif));
/// ```
pub fn is_native_for(ext: &str, target: EncodeTarget) -> bool {
  let allowed = match target {
    EncodeTarget::Avif => AVIF_NATIVE_EXTENSIONS,
    EncodeTarget::Jxl => JXL_NATIVE_EXTENSIONS,
  };
  allowed.iter().any(|ok| *ok == ext)
}

/// 判断这个扩展名要不要先转成 PNG
///
/// 必须是支持的图片，并且不是该编码器的原生格式。
///
/// # 参数
/// - `ext`: 小写扩展名
/// - `target`: 编码目标
///
/// # 返回值
/// 需要桥接时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(needs_bridge("webp", EncodeTarget::Avif));
/// assert!(!needs_bridge("png", EncodeTarget::Avif));
/// ```
pub fn needs_bridge(ext: &str, target: EncodeTarget) -> bool {
  is_supported_image_ext(ext) && !is_native_for(ext, target)
}

/// 把图片解码成临时 PNG，并在要求时写出 EXIF 载荷
///
/// 写入 PNG 或 EXIF 失败时删掉已经写出的 PNG。没有提出载荷时不创建 EXIF 文件。
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `input_path`: 源图片
/// - `extract_exif`: 是否提取 EXIF
///
/// # 返回值
/// 成功时返回桥接产物
///
/// # 错误
/// 解码失败返回 `SidecarError::Invalid`。取临时目录、写 PNG 或写 EXIF 失败返回 `SidecarError::Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let bridge = prepare_bridge_input(app, input_path, true)?;
/// ```
pub fn prepare_bridge_input(app: &AppHandle, input_path: &str, extract_exif: bool) -> SidecarResult<BridgeInput> {
  let temp_dir = get_cache_temp_dir(app)?;
  let id = new_uuid_v4();
  let png_path = temp_dir.join(format!("img-{id}.png"));
  let exif_candidate = temp_dir.join(format!("img-{id}.exif"));

  let img = image::open(input_path).map_err(|e| SidecarError::invalid(format!("桥接解码失败: {e}")))?;
  if let Err(e) = img.save(&png_path) {
    let _ = fs::remove_file(&png_path);
    return Err(SidecarError::io(format!("写入临时 PNG 失败: {e}")));
  }

  let exif_path = if extract_exif {
    match extract_exif_payload(input_path) {
      Some(bytes) if !bytes.is_empty() => match fs::write(&exif_candidate, &bytes) {
        Ok(()) => Some(exif_candidate),
        Err(e) => {
          let _ = fs::remove_file(&png_path);
          return Err(SidecarError::io(format!("写入临时 EXIF 失败: {e}")));
        }
      },
      _ => None,
    }
  } else {
    None
  };

  Ok(BridgeInput { png_path, exif_path })
}

/// 按扩展名从文件里取出 EXIF 载荷
///
/// JPEG 取 APP1 里 `Exif\0\0` 之后的字节。WebP 取 RIFF `EXIF` 块。PNG 取 `eXIf` 块。TIFF 重建只含元数据的小 TIFF。读失败、短于 8 字节，或其他扩展名时返回 `None`。
///
/// # 参数
/// - `path`: 图片路径
///
/// # 返回值
/// 提取到的载荷
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let payload = extract_exif_payload(path);
/// ```
fn extract_exif_payload(path: &str) -> Option<Vec<u8>> {
  let data = fs::read(path).ok()?;
  if data.len() < 8 {
    return None;
  }

  let ext = path_ext_lower(path).unwrap_or_default();
  match ext.as_str() {
    "jpg" | "jpeg" => extract_jpeg_exif(&data),
    "webp" => extract_webp_exif(&data),
    "png" => extract_png_exif(&data),
    "tif" | "tiff" => extract_tiff_exif(&data),
    _ => None,
  }
}

/// 从 JPEG 段里取出 APP1 EXIF 载荷
///
/// 开头不是 `FF D8` 时返回 `None`。跳过填充字节和独立标记，遇到 SOS 停止。APP1 以 `Exif\0\0` 开头时返回它后面的字节。
///
/// # 参数
/// - `data`: JPEG 字节
///
/// # 返回值
/// EXIF 载荷
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(extract_jpeg_exif(b"not-jpeg").is_none());
/// ```
fn extract_jpeg_exif(data: &[u8]) -> Option<Vec<u8>> {
  if data.len() < 4 || data[0] != 0xFF || data[1] != 0xD8 {
    return None;
  }
  let mut i = 2usize;
  while i + 4 <= data.len() {
    if data[i] != 0xFF {
      i += 1;
      continue;
    }
    // skip fill bytes
    while i < data.len() && data[i] == 0xFF {
      i += 1;
    }
    if i >= data.len() {
      break;
    }
    let marker = data[i];
    i += 1;
    // standalone markers
    if marker == 0xD8 || marker == 0xD9 || (0xD0..=0xD7).contains(&marker) {
      continue;
    }
    if i + 2 > data.len() {
      break;
    }
    let seg_len = u16::from_be_bytes([data[i], data[i + 1]]) as usize;
    if seg_len < 2 || i + seg_len > data.len() {
      break;
    }
    let seg = &data[i + 2..i + seg_len];
    if marker == 0xE1 && seg.len() >= 6 && &seg[..6] == b"Exif\0\0" {
      return Some(seg[6..].to_vec());
    }
    // SOS: stop scanning headers
    if marker == 0xDA {
      break;
    }
    i += seg_len;
  }
  None
}

/// 从 WebP 的 `EXIF` 块取出载荷
///
/// 不是 `RIFF`/`WEBP` 时返回 `None`。块大小按偶数补齐后继续。载荷以 `Exif\0\0` 开头时去掉这 6 个字节。
///
/// # 参数
/// - `data`: WebP 字节
///
/// # 返回值
/// EXIF 载荷
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(extract_webp_exif(b"not-webp").is_none());
/// ```
fn extract_webp_exif(data: &[u8]) -> Option<Vec<u8>> {
  if data.len() < 12 || &data[0..4] != b"RIFF" || &data[8..12] != b"WEBP" {
    return None;
  }
  let mut i = 12usize;
  while i + 8 <= data.len() {
    let fourcc = &data[i..i + 4];
    let size = u32::from_le_bytes([data[i + 4], data[i + 5], data[i + 6], data[i + 7]]) as usize;
    let payload_start = i + 8;
    let payload_end = payload_start.checked_add(size)?;
    if payload_end > data.len() {
      break;
    }
    if fourcc == b"EXIF" {
      let payload = &data[payload_start..payload_end];
      // 部分 WebP 在 EXIF chunk 内仍带 Exif\0\0 前缀
      if payload.len() >= 6 && &payload[..6] == b"Exif\0\0" {
        return Some(payload[6..].to_vec());
      }
      return Some(payload.to_vec());
    }
    // chunks are padded to even size
    i = payload_end + (size % 2);
  }
  None
}

/// 从 PNG 的 `eXIf` 块取出载荷
///
/// 签名不对时返回 `None`。遇到 `IEND` 停止。载荷以 `Exif\0\0` 开头时去掉这 6 个字节。
///
/// # 参数
/// - `data`: PNG 字节
///
/// # 返回值
/// EXIF 载荷
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(extract_png_exif(b"not-png").is_none());
/// ```
fn extract_png_exif(data: &[u8]) -> Option<Vec<u8>> {
  const SIG: &[u8] = &[137, 80, 78, 71, 13, 10, 26, 10];
  if data.len() < 8 || &data[..8] != SIG {
    return None;
  }
  let mut i = 8usize;
  while i + 12 <= data.len() {
    let len = u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;
    let ctype = &data[i + 4..i + 8];
    let payload_start = i + 8;
    let payload_end = payload_start.checked_add(len)?;
    // +4 CRC
    if payload_end + 4 > data.len() {
      break;
    }
    if ctype == b"eXIf" {
      let payload = &data[payload_start..payload_end];
      if payload.len() >= 6 && &payload[..6] == b"Exif\0\0" {
        return Some(payload[6..].to_vec());
      }
      return Some(payload.to_vec());
    }
    if ctype == b"IEND" {
      break;
    }
    i = payload_end + 4;
  }
  None
}

// --- TIFF → 元数据-only EXIF 载荷 ---

const TAG_EXIF_IFD: u16 = 0x8769;
const TAG_GPS_IFD: u16 = 0x8825;
const TAG_INTEROP_IFD: u16 = 0xA005;
const TAG_SUB_IFD: u16 = 0x014A;
const TAG_JPEG_IF_OFFSET: u16 = 0x0201;
const TAG_JPEG_IF_LENGTH: u16 = 0x0202;

/// TIFF IFD 里的一条元数据
///
/// 不超过 4 字节的值留在条目里，更长的值按偏移拷出。计数为 1 的 SHORT、LONG 或 IFD 才能读成 `u32`。
///
/// # 字段
/// - `tag`: TIFF 标签
/// - `type_`: 字段类型号
/// - `count`: 值的个数
/// - `value`: 值的字节。不超过 4 字节时是条目内的值，否则是按偏移拷出的字节
///
/// # 示例
/// ```ignore
/// let entry = TiffEntry { tag: 0x8769, type_: 4, count: 1, value: 8u32.to_le_bytes().to_vec() };
/// assert_eq!(entry_u32(&entry, true), Some(8));
/// ```
#[derive(Clone)]
struct TiffEntry {
  /// TIFF 标签
  tag: u16,
  /// 字段类型号
  type_: u16,
  /// 值的个数
  count: u32,
  /// 值的字节
  value: Vec<u8>,
}

/// 返回 TIFF 字段类型的单值字节数
///
/// 1、2、6、7 为 1 字节。3、8 为 2 字节。4、9、11、13 为 4 字节。5、10、12 为 8 字节。其他返回 `None`。
///
/// # 参数
/// - `type_`: TIFF 类型号
///
/// # 返回值
/// 单值字节数
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(tiff_type_size(3), Some(2));
/// assert_eq!(tiff_type_size(99), None);
/// ```
fn tiff_type_size(type_: u16) -> Option<usize> {
  match type_ {
    1 | 2 | 6 | 7 => Some(1),   // BYTE / ASCII / SBYTE / UNDEFINED
    3 | 8 => Some(2),           // SHORT / SSHORT
    4 | 9 | 11 | 13 => Some(4), // LONG / SLONG / FLOAT / IFD
    5 | 10 | 12 => Some(8),     // RATIONAL / SRATIONAL / DOUBLE
    _ => None,
  }
}

/// 判断这个 TIFF 标签是不是像素或子图像指针
///
/// 包括条带、瓦片、空闲区、SubIFD 和 JPEG 表。
///
/// # 参数
/// - `tag`: TIFF 标签
///
/// # 返回值
/// 属于这些标签时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(is_pixel_pointer_tag(0x0111));
/// assert!(!is_pixel_pointer_tag(0x0112));
/// ```
fn is_pixel_pointer_tag(tag: u16) -> bool {
  matches!(
    tag,
    0x0111 | // StripOffsets
    0x0116 | // RowsPerStrip
    0x0117 | // StripByteCounts
    0x0142 | // TileWidth
    0x0143 | // TileLength
    0x0144 | // TileOffsets
    0x0145 | // TileByteCounts
    0x0120 | // FreeOffsets
    0x0121 | // FreeByteCounts
    0x014A | // SubIFDs（其它图像 IFD，含像素）
    0x015B // JPEGTables（全图 JPEG 压缩表）
  )
}

/// 按字节序读取一个 u16
///
/// # 参数
/// - `data`: 字节
/// - `off`: 起点
/// - `le`: 为真时小端
///
/// # 返回值
/// 够两个字节时返回数值，否则 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(read_u16_e(&[0x2A, 0x00], 0, true), Some(42));
/// ```
fn read_u16_e(data: &[u8], off: usize, le: bool) -> Option<u16> {
  let b = data.get(off..off + 2)?;
  Some(if le { u16::from_le_bytes([b[0], b[1]]) } else { u16::from_be_bytes([b[0], b[1]]) })
}

/// 按字节序读取一个 u32
///
/// # 参数
/// - `data`: 字节
/// - `off`: 起点
/// - `le`: 为真时小端
///
/// # 返回值
/// 够四个字节时返回数值，否则 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(read_u32_e(&[8, 0, 0, 0], 0, true), Some(8));
/// ```
fn read_u32_e(data: &[u8], off: usize, le: bool) -> Option<u32> {
  let b = data.get(off..off + 4)?;
  Some(if le { u32::from_le_bytes([b[0], b[1], b[2], b[3]]) } else { u32::from_be_bytes([b[0], b[1], b[2], b[3]]) })
}

/// 按字节序追加一个 u16
///
/// # 参数
/// - `out`: 输出缓冲
/// - `v`: 数值
/// - `le`: 为真时小端
///
/// # 返回值
/// 无返回值
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let mut out = Vec::new();
/// write_u16_e(&mut out, 42, true);
/// assert_eq!(out, [0x2A, 0x00]);
/// ```
fn write_u16_e(out: &mut Vec<u8>, v: u16, le: bool) {
  if le {
    out.extend_from_slice(&v.to_le_bytes());
  } else {
    out.extend_from_slice(&v.to_be_bytes());
  }
}

/// 按字节序追加一个 u32
///
/// # 参数
/// - `out`: 输出缓冲
/// - `v`: 数值
/// - `le`: 为真时小端
///
/// # 返回值
/// 无返回值
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let mut out = Vec::new();
/// write_u32_e(&mut out, 8, true);
/// assert_eq!(out, [8, 0, 0, 0]);
/// ```
fn write_u32_e(out: &mut Vec<u8>, v: u32, le: bool) {
  if le {
    out.extend_from_slice(&v.to_le_bytes());
  } else {
    out.extend_from_slice(&v.to_be_bytes());
  }
}

/// 解析一个 IFD 里的条目和下一个 IFD 偏移
///
/// 条目超出数据、类型未知，或计数值乘类型长度溢出时返回 `None`。不超过 4 字节的值留在条目里，更长的值按偏移拷出。
///
/// # 参数
/// - `data`: TIFF 字节
/// - `ifd_off`: IFD 起点
/// - `le`: 为真时小端
///
/// # 返回值
/// 条目列表，以及下一个 IFD 的偏移
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(parse_ifd_entries(&[0, 0], 0, true).is_none());
/// ```
fn parse_ifd_entries(data: &[u8], ifd_off: usize, le: bool) -> Option<(Vec<TiffEntry>, u32)> {
  let count = read_u16_e(data, ifd_off, le)? as usize;
  let mut entries = Vec::with_capacity(count);
  let mut pos = ifd_off + 2;
  for _ in 0..count {
    if pos + 12 > data.len() {
      return None;
    }
    let tag = read_u16_e(data, pos, le)?;
    let type_ = read_u16_e(data, pos + 2, le)?;
    let cnt = read_u32_e(data, pos + 4, le)?;
    let type_sz = tiff_type_size(type_)?;
    let byte_len = (cnt as usize).checked_mul(type_sz)?;
    let value = if byte_len <= 4 {
      data.get(pos + 8..pos + 8 + byte_len)?.to_vec()
    } else {
      let val_off = read_u32_e(data, pos + 8, le)? as usize;
      let end = val_off.checked_add(byte_len)?;
      data.get(val_off..end)?.to_vec()
    };
    entries.push(TiffEntry { tag, type_, count: cnt, value });
    pos += 12;
  }
  let next = read_u32_e(data, pos, le)?;
  Some((entries, next))
}

/// 把计数为 1 的 SHORT、LONG 或 IFD 条目读成 u32
///
/// # 参数
/// - `entry`: TIFF 条目
/// - `le`: 为真时小端
///
/// # 返回值
/// 读到的数值。计数不是 1、类型不对或字节不够时返回 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let entry = TiffEntry { tag: 1, type_: 1, count: 2, value: vec![] };
/// assert!(entry_u32(&entry, true).is_none());
/// ```
fn entry_u32(entry: &TiffEntry, le: bool) -> Option<u32> {
  if entry.count != 1 {
    return None;
  }
  match entry.type_ {
    3 => {
      // SHORT
      if entry.value.len() < 2 {
        return None;
      }
      Some(read_u16_e(&entry.value, 0, le)? as u32)
    }
    4 | 13 => {
      if entry.value.len() < 4 {
        return None;
      }
      read_u32_e(&entry.value, 0, le)
    }
    _ => None,
  }
}

/// 去掉 IFD0 里的像素指针和 SubIFD
///
/// # 参数
/// - `entries`: 原始条目
///
/// # 返回值
/// 留下的条目
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(filter_ifd0_entries(Vec::new()).is_empty());
/// ```
fn filter_ifd0_entries(entries: Vec<TiffEntry>) -> Vec<TiffEntry> {
  entries.into_iter().filter(|e| !is_pixel_pointer_tag(e.tag) && e.tag != TAG_SUB_IFD).collect()
}

/// 从完整 TIFF 重建只含元数据的小 TIFF
///
/// 只接受字节序标记和魔数 42。去掉像素指针。Exif、GPS、Interop 子 IFD 有内容时把父指针收成 LONG 占位，没有则删掉悬空指针。IFD1 只在带 JPEG 缩略图时保留。没有任何元数据时返回 `None`。
///
/// # 参数
/// - `data`: TIFF 字节
///
/// # 返回值
/// 重建后的 TIFF
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(extract_tiff_exif(b"not-tiff").is_none());
/// ```
fn extract_tiff_exif(data: &[u8]) -> Option<Vec<u8>> {
  if data.len() < 8 {
    return None;
  }
  let le = match &data[0..2] {
    b"II" => true,
    b"MM" => false,
    _ => return None,
  };
  if read_u16_e(data, 2, le)? != 42 {
    return None;
  }
  let ifd0_off = read_u32_e(data, 4, le)? as usize;
  let (ifd0_raw, next_ifd) = parse_ifd_entries(data, ifd0_off, le)?;
  let mut ifd0 = filter_ifd0_entries(ifd0_raw);

  let exif_off = ifd0.iter().find(|e| e.tag == TAG_EXIF_IFD).and_then(|e| entry_u32(e, le));
  let gps_off = ifd0.iter().find(|e| e.tag == TAG_GPS_IFD).and_then(|e| entry_u32(e, le));

  let exif_entries =
    if let Some(off) = exif_off { parse_ifd_entries(data, off as usize, le).map(|(e, _)| e) } else { None };
  let gps_entries =
    if let Some(off) = gps_off { parse_ifd_entries(data, off as usize, le).map(|(e, _)| e) } else { None };

  // ExifIFD 内的 Interoperability IFD
  let interop_off = exif_entries
    .as_ref()
    .and_then(|ents| ents.iter().find(|e| e.tag == TAG_INTEROP_IFD))
    .and_then(|e| entry_u32(e, le));
  let interop_entries =
    if let Some(off) = interop_off { parse_ifd_entries(data, off as usize, le).map(|(e, _)| e) } else { None };

  // IFD1：仅在是 JPEG 缩略图时保留
  let thumb = if next_ifd > 0 {
    parse_ifd_entries(data, next_ifd as usize, le).and_then(|(ents, _)| {
      let jpeg_off = ents.iter().find(|e| e.tag == TAG_JPEG_IF_OFFSET).and_then(|e| entry_u32(e, le))?;
      let jpeg_len = ents.iter().find(|e| e.tag == TAG_JPEG_IF_LENGTH).and_then(|e| entry_u32(e, le))? as usize;
      if jpeg_len == 0 {
        return None;
      }
      let start = jpeg_off as usize;
      let end = start.checked_add(jpeg_len)?;
      let jpeg = data.get(start..end)?.to_vec();
      // 去掉指向全图像素的条目
      let thumb_ents: Vec<TiffEntry> = ents.into_iter().filter(|e| !is_pixel_pointer_tag(e.tag)).collect();
      if thumb_ents.iter().any(|e| e.tag == TAG_JPEG_IF_OFFSET) { Some((thumb_ents, jpeg)) } else { None }
    })
  } else {
    None
  };

  let has_meta = !ifd0.is_empty()
    || exif_entries.as_ref().is_some_and(|e| !e.is_empty())
    || gps_entries.as_ref().is_some_and(|e| !e.is_empty());
  if !has_meta {
    return None;
  }

  // 无对应子 IFD 时去掉悬空指针；有则规范为 LONG 占位以便回填偏移
  /// 把指定标签收成计数值为 1 的 LONG，数值先写 0
  ///
  /// 找不到该标签时不改。
  ///
  /// # 参数
  /// - `entries`: 条目
  /// - `tag`: 要改的标签
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let mut entries = Vec::new();
  /// ensure_long_ptr(&mut entries, 0x8769);
  /// ```
  fn ensure_long_ptr(entries: &mut [TiffEntry], tag: u16) {
    if let Some(e) = entries.iter_mut().find(|e| e.tag == tag) {
      e.type_ = 4;
      e.count = 1;
      e.value = vec![0, 0, 0, 0];
    }
  }
  if exif_entries.as_ref().is_some_and(|e| !e.is_empty()) {
    ensure_long_ptr(&mut ifd0, TAG_EXIF_IFD);
  } else {
    ifd0.retain(|e| e.tag != TAG_EXIF_IFD);
  }
  if gps_entries.as_ref().is_some_and(|e| !e.is_empty()) {
    ensure_long_ptr(&mut ifd0, TAG_GPS_IFD);
  } else {
    ifd0.retain(|e| e.tag != TAG_GPS_IFD);
  }

  let mut exif_entries = exif_entries;
  if let Some(ref mut ents) = exif_entries {
    if interop_entries.as_ref().is_some_and(|e| !e.is_empty()) {
      ensure_long_ptr(ents, TAG_INTEROP_IFD);
    } else {
      ents.retain(|e| e.tag != TAG_INTEROP_IFD);
    }
  }

  if ifd0.is_empty()
    && !exif_entries.as_ref().is_some_and(|e| !e.is_empty())
    && !gps_entries.as_ref().is_some_and(|e| !e.is_empty())
  {
    return None;
  }

  rebuild_exif_tiff(le, ifd0, exif_entries, gps_entries, interop_entries, thumb)
}

/// 用 0 把缓冲补到 4 字节对齐
///
/// # 参数
/// - `buf`: 输出缓冲
///
/// # 返回值
/// 无返回值
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let mut buf = vec![1, 2];
/// pad4(&mut buf);
/// assert_eq!(buf.len(), 4);
/// ```
fn pad4(buf: &mut Vec<u8>) {
  while buf.len() % 4 != 0 {
    buf.push(0);
  }
}

/// 写出一个 IFD，并返回稍后要回填的指针槽
///
/// 不超过 4 字节的值直接写入并补零。更长的值先占 4 字节，IFD 写完后把值追加到后面并回填偏移。Exif、GPS、Interop 和 JPEG 偏移标签即使值不超过 4 字节也记入待回填槽。
///
/// # 参数
/// - `out`: 输出缓冲
/// - `entries`: 条目
/// - `le`: 为真时小端
/// - `next_ifd`: 下一个 IFD 的偏移
///
/// # 返回值
/// `(标签, 回填起点)` 列表
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let mut out = Vec::new();
/// let slots = write_ifd_block(&mut out, &[], true, 0);
/// assert!(slots.is_empty());
/// ```
fn write_ifd_block(
  out: &mut Vec<u8>,
  entries: &[TiffEntry],
  le: bool,
  next_ifd: u32,
  // 写出后需回填的指针：(tag, 回填位置=条目 value/offset 字段起始)
) -> Vec<(u16, usize)> {
  let mut ptr_slots = Vec::new();
  write_u16_e(out, entries.len() as u16, le);
  for e in entries {
    write_u16_e(out, e.tag, le);
    write_u16_e(out, e.type_, le);
    write_u32_e(out, e.count, le);
    let slot = out.len();
    if e.value.len() <= 4 {
      out.extend_from_slice(&e.value);
      while out.len() < slot + 4 {
        out.push(0);
      }
    } else {
      // 占位，稍后写外部值并回填偏移
      write_u32_e(out, 0, le);
    }
    if matches!(e.tag, TAG_EXIF_IFD | TAG_GPS_IFD | TAG_INTEROP_IFD | TAG_JPEG_IF_OFFSET) {
      ptr_slots.push((e.tag, slot));
    } else if e.value.len() > 4 {
      ptr_slots.push((e.tag, slot));
    }
  }
  write_u32_e(out, next_ifd, le);

  // 外部值区（非 IFD 指针 / 非 JPEG 指针的大值）
  for e in entries {
    if e.value.len() > 4 && !matches!(e.tag, TAG_EXIF_IFD | TAG_GPS_IFD | TAG_INTEROP_IFD | TAG_JPEG_IF_OFFSET) {
      pad4(out);
      let off = out.len() as u32;
      out.extend_from_slice(&e.value);
      if let Some((_, slot)) = ptr_slots.iter().find(|(t, _)| *t == e.tag) {
        let bytes = if le { off.to_le_bytes() } else { off.to_be_bytes() };
        out[*slot..*slot + 4].copy_from_slice(&bytes);
      }
    }
  }
  ptr_slots
}

/// 按字节序把一个 u32 写进已有缓冲
///
/// # 参数
/// - `out`: 已有字节
/// - `slot`: 写入起点
/// - `v`: 数值
/// - `le`: 为真时小端
///
/// # 返回值
/// 无返回值
///
/// # Panics
/// `slot + 4` 超出 `out` 长度时，切片赋值会 panic
///
/// # 示例
/// ```
/// let mut out = [0, 0, 0, 0];
/// patch_u32(&mut out, 0, 8, true);
/// assert_eq!(out, [8, 0, 0, 0]);
/// ```
fn patch_u32(out: &mut [u8], slot: usize, v: u32, le: bool) {
  let bytes = if le { v.to_le_bytes() } else { v.to_be_bytes() };
  out[slot..slot + 4].copy_from_slice(&bytes);
}

/// 把 IFD0 和子 IFD 组装成一份 TIFF
///
/// 开头写字节序和魔数 42，IFD0 从偏移 8 开始。有 Exif、GPS、Interop 时对齐后写到后面，并把偏移回填到父指针。有 JPEG 缩略图时再写 IFD1，并回填 JPEG 偏移。
///
/// # 参数
/// - `le`: 为真时小端
/// - `ifd0`: IFD0 条目
/// - `exif`: Exif 子 IFD
/// - `gps`: GPS 子 IFD
/// - `interop`: Interop 子 IFD
/// - `thumb`: 缩略图条目和 JPEG 字节
///
/// # 返回值
/// 组装好的 TIFF
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let bytes = rebuild_exif_tiff(true, Vec::new(), None, None, None, None);
/// assert!(bytes.unwrap().starts_with(b"II"));
/// ```
fn rebuild_exif_tiff(
  le: bool,
  ifd0: Vec<TiffEntry>,
  exif: Option<Vec<TiffEntry>>,
  gps: Option<Vec<TiffEntry>>,
  interop: Option<Vec<TiffEntry>>,
  thumb: Option<(Vec<TiffEntry>, Vec<u8>)>,
) -> Option<Vec<u8>> {
  let mut out = Vec::with_capacity(256);
  if le {
    out.extend_from_slice(b"II");
  } else {
    out.extend_from_slice(b"MM");
  }
  write_u16_e(&mut out, 42, le);
  // IFD0 offset = 8
  write_u32_e(&mut out, 8, le);

  // next_ifd 稍后回填；先写 0
  let ifd0_slots = write_ifd_block(&mut out, &ifd0, le, 0);

  // 写子 IFD，并把偏移写回父 IFD 指针槽
  let write_sub_ifd =
    |out: &mut Vec<u8>, entries: &[TiffEntry], parent_slots: &[(u16, usize)], parent_tag: u16| -> Vec<(u16, usize)> {
      pad4(out);
      let off = out.len() as u32;
      if let Some((_, slot)) = parent_slots.iter().find(|(t, _)| *t == parent_tag) {
        patch_u32(out, *slot, off, le);
      }
      write_ifd_block(out, entries, le, 0)
    };

  let exif_slots =
    if let Some(ref ents) = exif { Some(write_sub_ifd(&mut out, ents, &ifd0_slots, TAG_EXIF_IFD)) } else { None };

  if let Some(ref ents) = gps {
    let _ = write_sub_ifd(&mut out, ents, &ifd0_slots, TAG_GPS_IFD);
  }

  if let (Some(ents), Some(parent_slots)) = (&interop, &exif_slots) {
    let _ = write_sub_ifd(&mut out, ents, parent_slots, TAG_INTEROP_IFD);
  }

  if let Some((thumb_ents, jpeg)) = thumb {
    pad4(&mut out);
    let thumb_ifd_off = out.len() as u32;
    // IFD0：count(2) + n*12 + next(4)；next 在外部值区之前
    let ifd0_next_slot = 8 + 2 + ifd0.len() * 12;
    patch_u32(&mut out, ifd0_next_slot, thumb_ifd_off, le);

    let thumb_slots = write_ifd_block(&mut out, &thumb_ents, le, 0);
    pad4(&mut out);
    let jpeg_off = out.len() as u32;
    out.extend_from_slice(&jpeg);
    if let Some((_, slot)) = thumb_slots.iter().find(|(t, _)| *t == TAG_JPEG_IF_OFFSET) {
      patch_u32(&mut out, *slot, jpeg_off, le);
    }
  }

  Some(out)
}

#[cfg(test)]
mod tests {
  use super::*;

  /// 构造最小 TIFF：IFD0 含 Orientation + ExifIFD，ExifIFD 含 DateTimeOriginal。
  fn sample_tiff_with_exif() -> Vec<u8> {
    let le = true;
    let mut buf = Vec::new();
    buf.extend_from_slice(b"II");
    buf.extend_from_slice(&42u16.to_le_bytes());
    buf.extend_from_slice(&8u32.to_le_bytes()); // IFD0 @ 8

    // IFD0: 2 entries — Orientation (0x0112 SHORT=1), ExifIFD (0x8769 LONG)
    // ExifIFD 将放在 IFD0 之后
    // IFD0 size = 2 + 2*12 + 4 = 30, so ends at 8+30=38
    // Put ASCII "2024:01:02 03:04:05\0" (20 bytes) after ExifIFD
    let exif_ifd_off = 38u32;
    buf.extend_from_slice(&2u16.to_le_bytes());
    // Orientation = 6
    buf.extend_from_slice(&0x0112u16.to_le_bytes());
    buf.extend_from_slice(&3u16.to_le_bytes()); // SHORT
    buf.extend_from_slice(&1u32.to_le_bytes());
    buf.extend_from_slice(&6u16.to_le_bytes());
    buf.extend_from_slice(&0u16.to_le_bytes());
    // ExifIFD pointer
    buf.extend_from_slice(&TAG_EXIF_IFD.to_le_bytes());
    buf.extend_from_slice(&4u16.to_le_bytes()); // LONG
    buf.extend_from_slice(&1u32.to_le_bytes());
    buf.extend_from_slice(&exif_ifd_off.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes()); // next IFD

    assert_eq!(buf.len(), 38);
    // ExifIFD: 1 entry — DateTimeOriginal 0x9003 ASCII
    let ascii = b"2024:01:02 03:04:05\0";
    let ascii_off = (38 + 2 + 12 + 4) as u32; // after exif IFD header+entry+next
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&0x9003u16.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes()); // ASCII
    buf.extend_from_slice(&(ascii.len() as u32).to_le_bytes());
    buf.extend_from_slice(&ascii_off.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(ascii);

    // Also append fake strip tags in a parallel structure? Not needed — test extraction.
    let _ = le;
    buf
  }

  #[test]
  fn tiff_exif_preserves_orientation_and_datetime() {
    let tiff = sample_tiff_with_exif();
    let payload = extract_tiff_exif(&tiff).expect("should extract");
    assert!(payload.starts_with(b"II"));
    // Orientation=6 应出现在载荷中（little-endian SHORT 6）
    assert!(payload.windows(2).any(|w| w == [6, 0]));
    // DateTimeOriginal ASCII
    assert!(payload.windows(19).any(|w| w == b"2024:01:02 03:04:05"));
  }

  #[test]
  fn tiff_strips_strip_offsets() {
    let data = sample_tiff_with_exif();
    let (mut ents, _) = parse_ifd_entries(&data, 8, true).unwrap();
    ents.push(TiffEntry { tag: 0x0111, type_: 4, count: 1, value: 0u32.to_le_bytes().to_vec() });
    let filtered = filter_ifd0_entries(ents);
    assert!(filtered.iter().all(|e| e.tag != 0x0111));
    assert!(filtered.iter().any(|e| e.tag == 0x0112));
  }
}
