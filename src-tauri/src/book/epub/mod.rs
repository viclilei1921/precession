//! EPUB：解析 container / OPF，按 spine 懒加载章节。

mod cfi;
mod content;
mod error;
mod meta;
mod nav;
mod package;
mod path;
mod smil;

use std::path::{Path, PathBuf};

use crate::book::types::{BookError, BookMeta, Chapter, ChapterRef, Format};

use content::assemble_epub_spine_entry;
use package::parse_package;

/// 已打开的 EPUB：缓存 OPF/spine，按章懒加载正文
pub struct EpubBook {
  path: PathBuf,
  meta: BookMeta,
  package: package::ParsedPackage,
}

impl EpubBook {
  /// 打开 EPUB：解析 container/OPF/导航一次并缓存
  pub fn open(path: &Path) -> Result<Self, BookError> {
    let package = parse_package(path).map_err(BookError::invalid)?;
    let meta =
      BookMeta { title: package.meta.title.clone(), author: package.meta.author.clone(), format: Format::Epub };
    Ok(Self { path: path.to_path_buf(), meta, package })
  }

  pub fn metadata(&self) -> &BookMeta {
    &self.meta
  }

  /// 仅目录标题（不读各章 xhtml）
  pub fn outline(&self) -> Vec<ChapterRef> {
    let mut titles: Vec<ChapterRef> = self
      .package
      .spine
      .iter()
      .enumerate()
      .map(|(i, s)| ChapterRef {
        index: i,
        title: if s.title.is_empty() { format!("第 {} 章", i + 1) } else { s.title.clone() },
      })
      .collect();
    if titles.is_empty() {
      titles.push(ChapterRef { index: 0, title: "正文".to_string() });
    }
    titles
  }

  /// 按 spine 索引读取单章（含图片与 CSS url 内联）
  pub fn chapter(&self, index: usize) -> Result<Chapter, BookError> {
    if self.package.spine.is_empty() {
      return if index == 0 {
        Ok(Chapter { index: 0, title: "正文".to_string(), content: String::new() })
      } else {
        Err(BookError::ChapterOutOfRange)
      };
    }
    let s = self.package.spine.get(index).ok_or(BookError::ChapterOutOfRange)?;
    let ch = assemble_epub_spine_entry(&self.path, s, index).map_err(BookError::invalid)?;
    Ok(Chapter { index, title: ch.title, content: ch.content })
  }

  /// 一次性读取全书各章
  pub fn content(self) -> Result<(BookMeta, Vec<Chapter>), BookError> {
    let mut chapters = Vec::new();
    if self.package.spine.is_empty() {
      chapters.push(Chapter { index: 0, title: "正文".to_string(), content: String::new() });
    } else {
      for i in 0..self.package.spine.len() {
        chapters.push(self.chapter(i)?);
      }
    }
    Ok((self.meta, chapters))
  }
}

pub(super) fn parse_metadata(path: &Path) -> Result<meta::EpubMeta, error::EpubError> {
  let package = parse_package(path)?;
  Ok(package.meta)
}
