//! 把系统选择器返回的地址收成可以打开的本地路径。
//!
//! Android 文件选择器交给页面的是 `content://` 地址。文档 ID 里的斜杠是 `%2F`，直接拿最后一段当书名会得到整段编码路径，而且 `File::open` 打不开这个地址。

use std::path::Path;

use super::error::FilesError;

/// 选择器路径在这一端该怎么用
///
/// # 变体
/// - `Ready`: 已经是本地路径，可以直接打开
/// - `Content`: Android content 地址，要先复制进缓存
pub enum Picked {
  /// 已经是本地路径，可以直接打开
  Ready(String),
  /// Android content 地址，要先复制进缓存
  Content(String),
}

/// 判断一条选择器路径要不要先复制
///
/// 空白返回 `Missing`。`content:` 留给 Android 复制。`file:` 解开百分号编码后，文件存在才当作本地路径。其余原样返回，不去检查文件在不在。
///
/// # 参数
/// - `path`: 选择器返回的路径或地址
///
/// # 返回值
/// 成功时返回 [`Picked`]
///
/// # 错误
/// 路径空白，或 `file:` 指向的文件不存在时返回 `FilesError::Missing`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let picked = classify("content://documents/document/raw%3A%2Fbook.epub")?;
/// ```
pub fn classify(path: &str) -> Result<Picked, FilesError> {
  let trimmed = path.trim();
  if trimmed.is_empty() {
    return Err(FilesError::Missing);
  }
  if is_content_uri(trimmed) {
    return Ok(Picked::Content(trimmed.to_string()));
  }
  if let Some(file) = file_uri_path(trimmed) {
    if Path::new(&file).is_file() {
      return Ok(Picked::Ready(file));
    }
    return Err(FilesError::Missing);
  }
  Ok(Picked::Ready(trimmed.to_string()))
}

/// 是不是 Android content 地址
///
/// 只看开头的 `content:`，忽略大小写。
///
/// # 参数
/// - `path`: 选择器返回的字符串
///
/// # 返回值
/// 以 `content:` 开头时返回 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// assert!(is_content_uri("content://documents/document/1"));
/// ```
fn is_content_uri(path: &str) -> bool {
  path.len() >= 8 && path[..8].eq_ignore_ascii_case("content:")
}

/// 把 `file:` 地址解开成本地路径
///
/// `file:///storage/a.epub` 变成 `/storage/a.epub`。百分号编码按 UTF-8 解开。不是 `file:` 时返回 `None`。
///
/// # 参数
/// - `raw`: 选择器返回的字符串
///
/// # 返回值
/// 能解开时返回本地路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let path = file_uri_path("file:///storage/%E4%B8%89.epub");
/// ```
fn file_uri_path(raw: &str) -> Option<String> {
  let rest = strip_file_scheme(raw)?;
  let decoded = percent_decode(rest);
  if decoded.starts_with('/') { Some(decoded) } else { None }
}

/// 去掉 `file://` 前缀
///
/// 大小写不敏感。不是这个前缀时返回 `None`。
///
/// # 参数
/// - `raw`: 原始字符串
///
/// # 返回值
/// 前缀后面的部分
///
/// # Panics
/// 不会主动 panic
fn strip_file_scheme(raw: &str) -> Option<&str> {
  const SCHEME: &str = "file://";
  if raw.len() >= SCHEME.len() && raw[..SCHEME.len()].eq_ignore_ascii_case(SCHEME) {
    Some(&raw[SCHEME.len()..])
  } else {
    None
  }
}

/// 解开百分号编码
///
/// `%` 后面不是两位十六进制时原样保留。解出来的字节按 UTF-8 解释，非法序列换成替换字符。
///
/// # 参数
/// - `input`: 可能含 `%XX` 的字符串
///
/// # 返回值
/// 解开后的文本
///
/// # Panics
/// 不会主动 panic
fn percent_decode(input: &str) -> String {
  let bytes = input.as_bytes();
  let mut out = Vec::with_capacity(bytes.len());
  let mut index = 0;
  while index < bytes.len() {
    if bytes[index] == b'%'
      && index + 2 < bytes.len()
      && let (Some(high), Some(low)) = (from_hex(bytes[index + 1]), from_hex(bytes[index + 2]))
    {
      out.push((high << 4) | low);
      index += 3;
      continue;
    }
    out.push(bytes[index]);
    index += 1;
  }
  String::from_utf8_lossy(&out).into_owned()
}

/// 把一个十六进制字符换成 0 到 15
///
/// 不是十六进制时返回 `None`。
///
/// # 参数
/// - `byte`: ASCII 字节
///
/// # 返回值
/// 对应的数值
///
/// # Panics
/// 不会主动 panic
fn from_hex(byte: u8) -> Option<u8> {
  match byte {
    b'0'..=b'9' => Some(byte - b'0'),
    b'a'..=b'f' => Some(byte - b'a' + 10),
    b'A'..=b'F' => Some(byte - b'A' + 10),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::{Picked, classify};

  #[test]
  fn content_uri_is_left_for_copy() {
    let uri = "content://com.android.externalstorage.documents/document/raw%3A%2Fstorage%2Femulated%2F0%2FDownload%2F%E4%B8%89%E4%BD%93.epub";
    match classify(uri).expect("classify") {
      Picked::Content(value) => assert_eq!(value, uri),
      Picked::Ready(_) => panic!("content uri should be copied"),
    }
  }

  #[test]
  fn file_uri_decodes_to_existing_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("三体.epub");
    std::fs::write(&file, b"pk").expect("write");
    let encoded = file.to_string_lossy().replace('三', "%E4%B8%89").replace('体', "%E4%BD%93");
    let uri = format!("file://{encoded}");
    match classify(&uri).expect("classify") {
      Picked::Ready(path) => assert_eq!(path, file.to_string_lossy()),
      Picked::Content(_) => panic!("file uri should be a local path"),
    }
  }

  #[test]
  fn plain_path_is_kept() {
    match classify("  /tmp/book.epub  ").expect("classify") {
      Picked::Ready(path) => assert_eq!(path, "/tmp/book.epub"),
      Picked::Content(_) => panic!("plain path"),
    }
  }

  #[test]
  fn blank_path_is_missing() {
    assert!(classify("   ").is_err());
  }
}
