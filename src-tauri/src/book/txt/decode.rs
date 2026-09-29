//! 读文件、编码探测与换行归一化

use std::fs;
use std::path::Path;

use crate::book::types::BookError;

/// BOM / UTF-16 / UTF-8 / GB18030 解码为 String
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

/// 归一化 txt 换行符
pub(super) fn normalize_txt_newlines(s: &str) -> String {
  s.replace("\r\n", "\n").replace('\r', "\n")
}

/// 读取并解码 txt 全文（换行已归一化）
pub(super) fn parse_txt_raw(path: &Path) -> Result<String, BookError> {
  let data = fs::read(path).map_err(BookError::invalid)?;
  Ok(normalize_txt_newlines(&decode_txt_bytes(&data)))
}
