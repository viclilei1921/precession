//! EPUB 章节正文组装（图片 / CSS url 内联为 data-URI）

use std::path::Path;

use base64::{Engine as _, engine::general_purpose};
use mime_guess::MimeGuess;
use regex::Regex;

use super::error::EpubError;
use super::meta::EpubChapter;
use super::package::{SpineEntry, read_zip_bytes, read_zip_text};
use super::path::{is_external_uri, resolve_url};

pub(crate) fn assemble_epub_spine_entry(path: &Path, s: &SpineEntry, index: usize) -> Result<EpubChapter, EpubError> {
  let raw = read_zip_text(path, &s.href).unwrap_or_default();
  let content = extract_body_or_raw(&raw);
  let content = inline_image_data_uris(path, &s.href, &content);
  let content = inline_css_url_data_uris(path, &s.href, &content);
  let title = if s.title.is_empty() { format!("第 {} 章", index + 1) } else { s.title.clone() };
  Ok(EpubChapter { title, content })
}

fn extract_body_or_raw(html: &str) -> String {
  let body_re = Regex::new(r"(?is)<body[^>]*>(.*?)</body>").expect("valid regex");
  if let Some(c) = body_re.captures(html) {
    return c.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
  }
  html.to_string()
}

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

/// 将 CSS 中的 url(...) 转为 base64 data URI
/// 覆盖 style 属性与 <style> 块中的 background-image、content 等
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

fn strip_url_suffix(path: &str) -> String {
  let no_hash = path.split('#').next().unwrap_or(path);
  no_hash.split('?').next().unwrap_or(no_hash).to_string()
}
