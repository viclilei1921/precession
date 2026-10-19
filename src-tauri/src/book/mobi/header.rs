//! MOBI 二进制读写与 PalmDoc 解压辅助

use base64::{Engine as _, engine::general_purpose};
use encoding_rs::{UTF_8, WINDOWS_1252};

use super::error::MobiError;

/// 读取大端 16 位整数
///
/// # 参数
/// - `b`: 至少含两个字节的切片，只使用前两字节
///
/// # 返回值
/// 大端 `u16`
///
/// # Panics
/// 切片长度不足 2 时下标访问会 panic
///
/// # 示例
/// ```
/// assert_eq!(read_u16_be(&[0x01, 0x02]), 0x0102);
/// ```
pub(crate) fn read_u16_be(b: &[u8]) -> u16 {
  u16::from_be_bytes([b[0], b[1]])
}

/// 读取大端 32 位整数
///
/// # 参数
/// - `b`: 至少含四个字节的切片，只使用前四字节
///
/// # 返回值
/// 大端 `u32`
///
/// # Panics
/// 切片长度不足 4 时下标访问会 panic
///
/// # 示例
/// ```
/// assert_eq!(read_u32_be(&[0x00, 0x00, 0x01, 0x00]), 256);
/// ```
pub(crate) fn read_u32_be(b: &[u8]) -> u32 {
  u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}

/// 从指定偏移读取大端 32 位整数
///
/// # 参数
/// - `data`: 源字节
/// - `offset`: 起始偏移，从 0 计
///
/// # 返回值
/// 从该处能取到 4 字节时返回整数，否则返回 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(read_u32_be_at(&[0x00, 0x00, 0x00, 0x01], 0), Some(1));
/// assert_eq!(read_u32_be_at(&[0x01], 0), None);
/// ```
pub(crate) fn read_u32_be_at(data: &[u8], offset: usize) -> Option<u32> {
  data.get(offset..offset + 4).map(read_u32_be)
}

/// 从指定偏移读取大端 16 位整数
///
/// # 参数
/// - `data`: 源字节
/// - `offset`: 起始偏移，从 0 计
///
/// # 返回值
/// 从该处能取到 2 字节时返回整数，否则返回 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(read_u16_be_at(&[0x01, 0x02, 0x03], 1), Some(0x0203));
/// assert_eq!(read_u16_be_at(&[0x01], 0), None);
/// ```
pub(crate) fn read_u16_be_at(data: &[u8], offset: usize) -> Option<u16> {
  data.get(offset..offset + 2).map(read_u16_be)
}

/// 按 MOBI 编码号选择解码器
///
/// # 参数
/// - `enc`: 编码号，`1252` 使用 Windows-1252
///
/// # 返回值
/// `1252` 返回 Windows-1252，其余返回 UTF-8
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(get_decoder(1252).name(), "windows-1252");
/// assert_eq!(get_decoder(65001).name(), "UTF-8");
/// ```
pub(crate) fn get_decoder(enc: u32) -> &'static encoding_rs::Encoding {
  match enc {
    1252 => WINDOWS_1252,
    _ => UTF_8,
  }
}

/// 统计字节里为 1 的位数
///
/// # 参数
/// - `x`: 待统计的字节
///
/// # 返回值
/// 最低位不断右移时遇到的 1 的个数
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(count_bits_set(0b1011), 3);
/// assert_eq!(count_bits_set(0), 0);
/// ```
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

/// 统计末尾连续的 0 位数
///
/// 从最低位向右移，直到遇到 1 或字节变成 0。
///
/// # 参数
/// - `x`: 待统计的字节
///
/// # 返回值
/// 末尾连续 0 的个数
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(count_unset_end(0b10100), 2);
/// assert_eq!(count_unset_end(0b1), 0);
/// ```
pub(crate) fn count_unset_end(mut x: u8) -> u8 {
  let mut count = 0;
  while x > 0 && x & 1 == 0 {
    x >>= 1;
    count += 1;
  }
  count
}

/// 从起点读取最多 4 字节的变长整数
///
/// 每字节取低 7 位左移累加。遇到高位为 1 的字节就结束并返回值和已读长度。4 字节内都没有高位时返回 `None`。起点之后不足 4 字节则只看到剩余部分。
///
/// # 参数
/// - `data`: 源字节
/// - `start`: 起始偏移，从 0 计
///
/// # 返回值
/// 成功返回数值和消耗的字节数
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(get_var_len(&[0x81], 0), Some((1, 1)));
/// assert_eq!(get_var_len(&[0x01, 0x02], 0), None);
/// ```
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

/// 从末尾最多 4 字节还原变长整数
///
/// 只查看最后 4 字节。高位为 1 时先把累计值清零，再并入该字节的低 7 位。
///
/// # 参数
/// - `data`: 源字节
///
/// # 返回值
/// 还原出的整数；空切片返回 0
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(get_var_len_from_end(&[0x81]), 1);
/// assert_eq!(get_var_len_from_end(&[0x01, 0x82]), 2);
/// ```
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

/// 按 PalmDOC 规则解压字节
///
/// `0` 原样写入。`1` 到 `8` 表示随后原样复制的字节数，超出源长度则只复制剩余部分。`0x09` 到 `0x7f` 原样写入。`0x80` 到 `0xbf` 与下一字节组成距离和长度并回抄已输出内容，距离越界的位置不写入；源里不够两字节时停止。其余字节写成空格再加上该字节去掉最高位。
///
/// # 参数
/// - `src`: 压缩后的字节
///
/// # 返回值
/// 解压后的字节
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// assert_eq!(decompress_palmdoc(b"Hi"), b"Hi");
/// assert_eq!(decompress_palmdoc(&[0x01, b'A']), b"A");
/// ```
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

/// 根据文件头判断图片 MIME
///
/// 长度不足 4 返回 `None`。依次识别 JPEG、PNG、GIF、BMP、WEBP、以 `<svg` 或 `<?xml` 开头的 SVG。
///
/// # 参数
/// - `data`: 图片字节
///
/// # 返回值
/// 识别成功返回 MIME 字符串，否则返回 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(detect_image_mime(&[0xFF, 0xD8, 0xFF, 0x00]), Some("image/jpeg"));
/// assert_eq!(detect_image_mime(b"\x89PNG...."), Some("image/png"));
/// assert_eq!(detect_image_mime(b"txt"), None);
/// ```
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

/// 把字节编成 data URI
///
/// # 参数
/// - `mime`: MIME 类型
/// - `data`: 原始字节
///
/// # 返回值
/// `data:{mime};base64,{编码}` 形式的字符串
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(make_data_uri("text/plain", b"hi"), "data:text/plain;base64,aGk=");
/// ```
pub(crate) fn make_data_uri(mime: &str, data: &[u8]) -> String {
  format!("data:{};base64,{}", mime, general_purpose::STANDARD.encode(data))
}

/// 判断 MIME 是否按文本资源处理
///
/// # 参数
/// - `mime`: MIME 类型
///
/// # 返回值
/// 属于 XML、XHTML、HTML、CSS 或 SVG 时返回 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(is_textual_resource_mime("text/html"));
/// assert!(!is_textual_resource_mime("image/jpeg"));
/// ```
pub(crate) fn is_textual_resource_mime(mime: &str) -> bool {
  matches!(
    mime,
    "application/xml" | "application/xhtml+xml" | "text/html" | "text/css" | "image/svg+xml"
  )
}

/// 按编码号把字节解码成字符串
///
/// # 参数
/// - `data`: 待解码字节
/// - `enc`: 交给解码器选择的编码号
///
/// # 返回值
/// 始终返回 `Ok`，内容是解码后的字符串
///
/// # 错误
/// 当前实现不会返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(decode_bytes(b"hi", 65001).unwrap(), "hi");
/// ```
pub(crate) fn decode_bytes(data: &[u8], enc: u32) -> Result<String, MobiError> {
  let (s, _, _) = get_decoder(enc).decode(data);
  Ok(s.into_owned())
}

/// 按编码号解码 EXTH 字符串
///
/// 空字节直接返回空字符串，否则用对应解码器解码。
///
/// # 参数
/// - `data`: EXTH 记录内容
/// - `enc`: 交给解码器选择的编码号
///
/// # 返回值
/// 始终返回 `Ok`，内容是解码后的字符串
///
/// # 错误
/// 当前实现不会返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(decode_exth_string(b"", 65001).unwrap(), "");
/// assert_eq!(decode_exth_string(b"hi", 65001).unwrap(), "hi");
/// ```
pub(crate) fn decode_exth_string(data: &[u8], enc: u32) -> Result<String, MobiError> {
  if data.is_empty() {
    return Ok(String::new());
  }
  let (s, _, _) = get_decoder(enc).decode(data);
  Ok(s.into_owned())
}
