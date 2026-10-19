//! 读文件、编码探测与换行归一化

use std::fs;
use std::path::Path;

use crate::book::types::BookError;

/// 把 TXT 字节解码成字符串
///
/// 去掉 UTF-8 BOM 后，按 UTF-16 LE、UTF-16 BE、UTF-8、GB18030 的顺序尝试。UTF-16 会再跳过前两个字节。
///
/// # 参数
/// - `data`: 文件原始字节
///
/// # 返回值
/// 解码后的字符串
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(decode_txt_bytes(b"\xEF\xBB\xBFhi"), "hi");
/// assert_eq!(decode_txt_bytes(b"hello"), "hello");
/// ```
pub(super) fn decode_txt_bytes(data: &[u8]) -> String {
  let slice = if data.starts_with(&[0xEF, 0xBB, 0xBF]) { &data[3..] } else { data };
  if slice.len() >= 2 && slice[0] == 0xFF && slice[1] == 0xFE {
    return encoding_rs::UTF_16LE.decode(&slice[2..]).0.into_owned();
  }
  if slice.len() >= 2 && slice[0] == 0xFE && slice[1] == 0xFF {
    return encoding_rs::UTF_16BE.decode(&slice[2..]).0.into_owned();
  }
  if let Ok(s) = std::str::from_utf8(slice) {
    return s.to_string();
  }
  encoding_rs::GB18030.decode(slice).0.into_owned()
}

/// 把换行符统一成 `\n`
///
/// 先把 `\r\n` 换成 `\n`，再把剩余的 `\r` 换成 `\n`。
///
/// # 参数
/// - `s`: 原始文本
///
/// # 返回值
/// 换行已统一的新字符串
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(normalize_txt_newlines("a\r\nb\rc"), "a\nb\nc");
/// ```
pub(super) fn normalize_txt_newlines(s: &str) -> String {
  s.replace("\r\n", "\n").replace('\r', "\n")
}

/// 读取 TXT 文件并解码全文
///
/// 读入全部字节后解码，再把换行统一成 `\n`。
///
/// # 参数
/// - `path`: TXT 文件路径
///
/// # 返回值
/// 成功返回解码并换行归一化后的全文
///
/// # 错误
/// 读文件失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let text = parse_txt_raw(Path::new("book.txt")).unwrap();
/// assert!(!text.contains('\r'));
/// ```
pub(super) fn parse_txt_raw(path: &Path) -> Result<String, BookError> {
  let data = fs::read(path).map_err(BookError::invalid)?;
  Ok(normalize_txt_newlines(&decode_txt_bytes(&data)))
}
