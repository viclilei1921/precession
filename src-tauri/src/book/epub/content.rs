//! EPUB 章节正文组装（图片 / CSS url 内联为 data-URI）

use std::path::Path;

use base64::{Engine as _, engine::general_purpose};
use mime_guess::MimeGuess;
use regex::Regex;

use super::error::EpubError;
use super::meta::EpubChapter;
use super::package::{SpineEntry, read_zip_bytes, read_zip_text};
use super::path::{is_external_uri, resolve_url};

/// 组装 spine 条目对应的一章
///
/// 读取压缩包里的章节文本失败时按空字符串继续。依次取出 body、把图片和 CSS `url` 内联成 data URI。标题为空时写成 `第 {n} 章`，序号从 1 开始。
///
/// # 参数
/// - `path`: EPUB 文件路径
/// - `s`: 已解析的 spine 条目
/// - `index`: 从 0 开始的章节下标
///
/// # 返回值
/// 始终返回 `Ok`，内容是标题和处理后的正文
///
/// # 错误
/// 当前实现不会返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let entry = SpineEntry { href: "c.xhtml".to_string(), title: "章".to_string() };
/// let chapter = assemble_epub_spine_entry(Path::new("book.epub"), &entry, 0).unwrap();
/// assert_eq!(chapter.title, "章");
/// ```
pub(crate) fn assemble_epub_spine_entry(path: &Path, s: &SpineEntry, index: usize) -> Result<EpubChapter, EpubError> {
  let raw = read_zip_text(path, &s.href).unwrap_or_default();
  let content = extract_body_or_raw(&raw);
  let content = inline_image_data_uris(path, &s.href, &content);
  let content = inline_css_url_data_uris(path, &s.href, &content);
  let title = if s.title.is_empty() { format!("第 {} 章", index + 1) } else { s.title.clone() };
  Ok(EpubChapter { title, content })
}

/// 取出 body 内部，没有 body 时返回原文
///
/// # 参数
/// - `html`: 章节 HTML
///
/// # 返回值
/// `body` 内部文本，或原始 HTML
///
/// # Panics
/// 正则编译失败时 `expect` 会 panic
///
/// # 示例
/// ```
/// assert_eq!(extract_body_or_raw("<body>hi</body>"), "hi");
/// assert_eq!(extract_body_or_raw("<p>hi</p>"), "<p>hi</p>");
/// ```
fn extract_body_or_raw(html: &str) -> String {
  let body_re = Regex::new(r"(?is)<body[^>]*>(.*?)</body>").expect("valid regex");
  if let Some(c) = body_re.captures(html) {
    return c.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
  }
  html.to_string()
}

/// 把 img 的 src 内联成 data URI
///
/// 空地址、`data:`、外部 URI 和 `javascript:` 保持原标签。其余地址按章节路径解析并去掉片段和查询后，从 EPUB 读取字节，猜 MIME，写成 `src="data:..."`。读不到时保留原标签。
///
/// # 参数
/// - `book_path`: EPUB 文件路径
/// - `chapter_href`: 当前章节路径，用来解析相对地址
/// - `html`: 章节 HTML
///
/// # 返回值
/// 替换过图片地址的 HTML
///
/// # Panics
/// 正则编译失败时 `expect` 会 panic
///
/// # 示例
/// ```ignore
/// let html = inline_image_data_uris(Path::new("book.epub"), "c.xhtml", "<img src=\"a.png\">");
/// assert!(html.contains("src="));
/// ```
fn inline_image_data_uris(book_path: &Path, chapter_href: &str, html: &str) -> String {
  let img_tag_re = Regex::new(r"(?is)<img\b[^>]*>").expect("valid regex");
  let src_re = Regex::new(r#"(?is)\bsrc\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#).expect("valid regex");

  let mut out = String::with_capacity(html.len());
  let mut last_end = 0usize;

  for m in img_tag_re.find_iter(html) {
    out.push_str(&html[last_end..m.start()]);
    let tag = m.as_str();

    let replaced = if let Some(c) = src_re.captures(tag) {
      let src = c.get(1).or_else(|| c.get(2)).or_else(|| c.get(3)).map(|x| x.as_str()).unwrap_or_default();

      if src.is_empty() || src.starts_with("data:") || is_external_uri(src) || src.starts_with("javascript:") {
        tag.to_string()
      } else {
        let resolved = resolve_url(src, chapter_href);
        let entry = strip_url_suffix(&resolved);
        if let Ok(bytes) = read_zip_bytes(book_path, &entry) {
          let mime = MimeGuess::from_path(&entry).first_or_octet_stream().essence_str().to_string();
          let b64 = general_purpose::STANDARD.encode(bytes);
          let data_uri = format!("data:{};base64,{}", mime, b64);
          src_re.replace(tag, format!(r#"src="{}""#, data_uri)).to_string()
        } else {
          tag.to_string()
        }
      }
    } else {
      tag.to_string()
    };

    out.push_str(&replaced);
    last_end = m.end();
  }

  out.push_str(&html[last_end..]);
  out
}

/// 把 CSS `url(...)` 内联成 data URI
///
/// 空路径、`data:`、外部 URI 和 `javascript:` 保持原样。其余路径按章节路径解析并去掉片段和查询后，从 EPUB 读取字节并写成 `url("data:...")`。读不到时保留原来的 `url(...)`。
///
/// # 参数
/// - `book_path`: EPUB 文件路径
/// - `chapter_href`: 当前章节路径，用来解析相对地址
/// - `html`: 含 CSS `url` 的 HTML
///
/// # 返回值
/// 替换过 `url(...)` 的 HTML
///
/// # Panics
/// 正则编译失败时 `expect` 会 panic；捕获组下标 `caps[0]` 缺失时会 panic
///
/// # 示例
/// ```ignore
/// let html = inline_css_url_data_uris(Path::new("book.epub"), "c.xhtml", "url(a.png)");
/// assert!(html.contains("url("));
/// ```
fn inline_css_url_data_uris(book_path: &Path, chapter_href: &str, html: &str) -> String {
  let url_re = Regex::new(r#"(?i)url\s*\(\s*["']?([^"')]+)["']?\s*\)"#).expect("valid regex");

  url_re
    .replace_all(html, |caps: &regex::Captures<'_>| {
      let path = caps.get(1).map(|m| m.as_str()).unwrap_or_default().trim();
      if path.is_empty() || path.starts_with("data:") || is_external_uri(path) || path.starts_with("javascript:") {
        caps[0].to_string()
      } else {
        let resolved = resolve_url(path, chapter_href);
        let entry = strip_url_suffix(&resolved);
        match read_zip_bytes(book_path, &entry) {
          Ok(bytes) => {
            let mime = MimeGuess::from_path(&entry).first_or_octet_stream().essence_str().to_string();
            let b64 = general_purpose::STANDARD.encode(bytes);
            let data_uri = format!("data:{};base64,{}", mime, b64);
            format!(r#"url("{}")"#, data_uri)
          }
          Err(_) => caps[0].to_string(),
        }
      }
    })
    .to_string()
}

/// 去掉路径里的片段和查询
///
/// 先截到第一个 `#` 之前，再截到第一个 `?` 之前。
///
/// # 参数
/// - `path`: 可能带 `#` 或 `?` 的路径
///
/// # 返回值
/// 去掉后缀后的路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(strip_url_suffix("a/b.xhtml#id?x=1"), "a/b.xhtml");
/// assert_eq!(strip_url_suffix("a/b.xhtml?x=1#id"), "a/b.xhtml");
/// ```
fn strip_url_suffix(path: &str) -> String {
  let no_hash = path.split('#').next().unwrap_or(path);
  no_hash.split('?').next().unwrap_or(no_hash).to_string()
}
