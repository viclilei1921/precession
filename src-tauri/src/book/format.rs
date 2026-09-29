//! 电子书格式探测：扩展名优先，失败则按文件头嗅探

use std::fs;
use std::io::Read;
use std::path::Path;

use super::types::Format;

/// 从路径解析格式（扩展名优先，失败则按文件头嗅探）
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

/// 按文件头嗅探：ZIP → epub，PDB type BOOKMOBI/TEXtREAd → mobi
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
