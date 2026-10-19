//! 电子书格式探测：扩展名优先，失败则按文件头嗅探

use std::fs;
use std::io::Read;
use std::path::Path;

use super::types::Format;

/// 按路径判断电子书格式
///
/// 先看小写扩展名：`epub` 为 EPUB，`mobi`、`azw`、`azw3` 为 MOBI，`txt` 为 TXT。扩展名不认识时再按文件头嗅探。
///
/// # 参数
/// - `path`: 书籍文件路径
///
/// # 返回值
/// 识别成功返回格式，否则返回 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let format = detect_format(Path::new("novel.epub"));
/// assert_eq!(format, Some(Format::Epub));
/// ```
pub fn detect_format(path: &Path) -> Option<Format> {
  if let Some(ext) = path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()) {
    match ext.as_str() {
      "epub" => return Some(Format::Epub),
      "mobi" | "azw" | "azw3" => return Some(Format::Mobi),
      "txt" => return Some(Format::Txt),
      _ => {}
    }
  }
  detect_format_by_sniff(path)
}

/// 按文件头嗅探电子书格式
///
/// 读取最多 68 字节。以 `PK` 压缩头开头视为 EPUB；偏移 60 处为 `BOOKMOBI` 或 `TEXtREAd` 视为 MOBI。打不开文件或读失败时返回 `None`。
///
/// # 参数
/// - `path`: 书籍文件路径
///
/// # 返回值
/// 识别成功返回格式，否则返回 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let format = detect_format_by_sniff(Path::new("book.bin"));
/// assert!(format.is_none() || matches!(format, Some(Format::Epub | Format::Mobi)));
/// ```
fn detect_format_by_sniff(path: &Path) -> Option<Format> {
  let mut file = fs::File::open(path).ok()?;
  let mut buf = [0u8; 68];
  let n = file.read(&mut buf).ok()?;
  let data = &buf[..n];
  if data.starts_with(b"PK\x03\x04") || data.starts_with(b"PK\x05\x06") || data.starts_with(b"PK\x07\x08") {
    return Some(Format::Epub);
  }
  if data.len() >= 68 && (&data[60..68] == b"BOOKMOBI" || &data[60..68] == b"TEXtREAd") {
    return Some(Format::Mobi);
  }
  None
}
