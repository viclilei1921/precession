//! 纯文本电子书：编码探测、书名片头与章节切分

mod decode;
mod heading;
mod meta;
mod number;
mod split;

use std::path::Path;

use crate::book::types::{BookError, BookMeta, Chapter, ChapterRef};

use decode::parse_txt_raw;
use heading::extract_chapter_title;
use meta::{extract_author, extract_title};
use split::split_txt_book;

/// 已打开的 TXT：打开时解码并分章，正文缓存在内存
pub struct TxtBook {
  meta: BookMeta,
  chapters: Vec<Chapter>,
}

impl TxtBook {
  /// 打开并解析 TXT（UTF-8 / UTF-16 / GB18030）
  pub fn open(path: &Path) -> Result<Self, BookError> {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("未知标题");
    let text = parse_txt_raw(path)?;
    let (meta, chapters) = split_txt_book(&text, stem);
    Ok(Self { meta, chapters })
  }

  /// 获取书籍元数据
  pub fn metadata(&self) -> &BookMeta {
    &self.meta
  }

  /// 获取书籍目录
  pub fn outline(&self) -> Vec<ChapterRef> {
    self.chapters.iter().map(|c| ChapterRef { index: c.index, title: c.title.clone() }).collect()
  }

  /// 获取指定章节
  pub fn chapter(&self, index: usize) -> Result<Chapter, BookError> {
    self.chapters.get(index).cloned().ok_or(BookError::ChapterOutOfRange)
  }

  /// 获取全书内容
  pub fn content(self) -> Result<(BookMeta, Vec<Chapter>), BookError> {
    Ok((self.meta, self.chapters))
  }
}

/// 仅元数据（上传前探测）
pub fn parse_metadata(path: &Path) -> Result<BookMeta, BookError> {
  let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("未知标题");

  let text = parse_txt_raw(path)?;
  let head: String = text.chars().take(8192).collect();
  let mut meta = BookMeta { title: String::new(), author: String::new(), format: crate::book::types::Format::Txt };

  for (i, line) in head.lines().enumerate() {
    if extract_chapter_title(line).is_some() {
      break;
    }
    if meta.title.is_empty() {
      if let Some(title) = extract_title(line) {
        meta.title = title;
      }
    }
    if meta.author.is_empty() {
      if let Some(author) = extract_author(line) {
        meta.author = author;
      }
    }
    if i > 12 {
      break;
    }
  }

  if meta.title.is_empty() {
    meta.title = stem.to_string();
  }
  Ok(meta)
}
