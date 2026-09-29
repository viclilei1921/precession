//! MOBI 二进制读写与 PalmDoc 解压辅助

use base64::{Engine as _, engine::general_purpose};
use encoding_rs::{UTF_8, WINDOWS_1252};

use super::error::MobiError;

pub(crate) fn read_u16_be(b: &[u8]) -> u16 {
  u16::from_be_bytes([b[0], b[1]])
}

pub(crate) fn read_u32_be(b: &[u8]) -> u32 {
  u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}

pub(crate) fn read_u32_be_at(data: &[u8], offset: usize) -> Option<u32> {
  data.get(offset..offset + 4).map(read_u32_be)
}

pub(crate) fn read_u16_be_at(data: &[u8], offset: usize) -> Option<u16> {
  data.get(offset..offset + 2).map(read_u16_be)
}

pub(crate) fn get_decoder(enc: u32) -> &'static encoding_rs::Encoding {
  match enc {
    1252 => WINDOWS_1252,
    _ => UTF_8,
  }
}

pub(crate) fn count_bits_set(mut x: u8) -> u8 {
  let mut count = 0;
  while x > 0 {
    if x & 1 == 1 {
      count += 1;
    }
    x >>= 1;
  }
  count
}

pub(crate) fn count_unset_end(mut x: u8) -> u8 {
  let mut count = 0;
  while x > 0 && x & 1 == 0 {
    x >>= 1;
    count += 1;
  }
  count
}

pub(crate) fn get_var_len(data: &[u8], start: usize) -> Option<(u32, usize)> {
  let mut value = 0u32;
  let mut length = 0usize;
  for byte in data.get(start..start + 4).unwrap_or(&[]) {
    value = (value << 7) | (u32::from(*byte & 0x7f));
    length += 1;
    if byte & 0x80 != 0 {
      return Some((value, length));
    }
  }
  None
}

/// 正向遍历末尾最多 4 字节，高位 (0x80) 标记编码值的起始位置。
pub(crate) fn get_var_len_from_end(data: &[u8]) -> u32 {
  let start = data.len().saturating_sub(4);
  let mut value: u32 = 0;
  for &byte in &data[start..] {
    if byte & 0x80 != 0 {
      value = 0;
    }
    value = (value << 7) | (byte & 0x7f) as u32;
  }
  value
}

/// PalmDOC 解压（compression=2）
pub(crate) fn decompress_palmdoc(src: &[u8]) -> Vec<u8> {
  let mut out = Vec::with_capacity(src.len() * 2);
  let mut i = 0;
  while i < src.len() {
    let b = src[i];
    if b == 0 {
      out.push(0);
      i += 1;
    } else if b <= 8 {
      let n = b as usize;
      i += 1;
      let end = (i + n).min(src.len());
      out.extend_from_slice(&src[i..end]);
      i = end;
    } else if b <= 0x7f {
      out.push(b);
      i += 1;
    } else if b <= 0xbf {
      if i + 2 > src.len() {
        break;
      }
      let word = (b as u16) << 8 | src[i + 1] as u16;
      let distance = ((word & 0x3FFF) >> 3) as usize;
      let length = ((word & 7) + 3) as usize;
      for _ in 0..length {
        if distance > 0 && distance <= out.len() {
          let idx = out.len() - distance;
          out.push(out[idx]);
        }
      }
      i += 2;
    } else {
      out.push(0x20);
      out.push(b ^ 0x80);
      i += 1;
    }
  }
  out
}

pub(crate) fn detect_image_mime(data: &[u8]) -> Option<&'static str> {
  if data.len() < 4 {
    return None;
  }
  if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
    Some("image/jpeg")
  } else if data.starts_with(b"\x89PNG") {
    Some("image/png")
  } else if data.starts_with(b"GIF8") {
    Some("image/gif")
  } else if data.starts_with(b"BM") {
    Some("image/bmp")
  } else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WEBP" {
    Some("image/webp")
  } else if data.starts_with(b"<svg") || data.starts_with(br#"<?xml"#) {
    Some("image/svg+xml")
  } else {
    None
  }
}

pub(crate) fn make_data_uri(mime: &str, data: &[u8]) -> String {
  format!("data:{};base64,{}", mime, general_purpose::STANDARD.encode(data))
}

pub(crate) fn is_textual_resource_mime(mime: &str) -> bool {
  matches!(
    mime,
    "application/xml" | "application/xhtml+xml" | "text/html" | "text/css" | "image/svg+xml"
  )
}

pub(crate) fn decode_bytes(data: &[u8], enc: u32) -> Result<String, MobiError> {
  let (s, _, _) = get_decoder(enc).decode(data);
  Ok(s.into_owned())
}

pub(crate) fn decode_exth_string(data: &[u8], enc: u32) -> Result<String, MobiError> {
  if data.is_empty() {
    return Ok(String::new());
  }
  let (s, _, _) = get_decoder(enc).decode(data);
  Ok(s.into_owned())
}
