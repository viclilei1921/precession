//! 本地电子书解析。支持 EPUB、MOBI、TXT。
//!
//! 只读文件、拆章节，不建表，也不走档案库。书和书摘仍在 `library`。
//! 阅读入口还没接到书库。接上之后去掉这条 allow，让没用到的解析代码重新报警。
#![allow(dead_code)]

mod epub;
mod format;
mod mobi;
mod txt;
mod types;

use std::path::Path;

pub use format::detect_format;
pub use types::{BookError, BookMeta, Chapter, ChapterRef, Format};

use epub::EpubBook;
use mobi::MobiBook;
use txt::TxtBook;

/// 已打开的电子书（枚举门面，对齐各格式的 outline / chapter API）
pub enum OpenedBook {
  Epub(EpubBook),
  Mobi(MobiBook),
  Txt(TxtBook),
}

impl OpenedBook {
  /// 按路径探测格式并打开；文件须可读
  pub fn open(path: &Path) -> Result<Self, BookError> {
    let format = detect_format(path).ok_or(BookError::Unsupported)?;
    Self::open_with_format(path, format)
  }

  /// 使用已知格式打开（跳过探测）
  pub fn open_with_format(path: &Path, format: Format) -> Result<Self, BookError> {
    if !path.exists() {
      return Err(BookError::NotFound);
    }
    match format {
      Format::Epub => Ok(OpenedBook::Epub(EpubBook::open(path)?)),
      Format::Mobi => Ok(OpenedBook::Mobi(MobiBook::open(path)?)),
      Format::Txt => Ok(OpenedBook::Txt(TxtBook::open(path)?)),
    }
  }

  pub fn metadata(&self) -> &BookMeta {
    match self {
      OpenedBook::Epub(b) => b.metadata(),
      OpenedBook::Mobi(b) => b.metadata(),
      OpenedBook::Txt(b) => b.metadata(),
    }
  }

  /// 仅目录（标题列表）
  pub fn outline(&self) -> Vec<ChapterRef> {
    match self {
      OpenedBook::Epub(b) => b.outline(),
      OpenedBook::Mobi(b) => b.outline(),
      OpenedBook::Txt(b) => b.outline(),
    }
  }

  /// 按索引取单章正文
  pub fn chapter(&self, index: usize) -> Result<Chapter, BookError> {
    match self {
      OpenedBook::Epub(b) => b.chapter(index),
      OpenedBook::Mobi(b) => b.chapter(index),
      OpenedBook::Txt(b) => b.chapter(index),
    }
  }

  /// 一次性取出全书各章（兼容旧「整本解析」调用）
  pub fn content(self) -> Result<(BookMeta, Vec<Chapter>), BookError> {
    match self {
      OpenedBook::Epub(b) => b.content(),
      OpenedBook::Mobi(b) => b.content(),
      OpenedBook::Txt(b) => b.content(),
    }
  }

  /// 仅解析元数据（不建立完整章节缓存；EPUB 仍会解析 OPF）
  pub fn parse_metadata(path: &Path) -> Result<BookMeta, BookError> {
    let format = detect_format(path).ok_or(BookError::Unsupported)?;
    match format {
      Format::Epub => {
        let m = epub::parse_metadata(path).map_err(BookError::invalid)?;
        Ok(BookMeta { title: m.title, author: m.author, format: Format::Epub })
      }
      Format::Mobi => {
        let m = mobi::parse_metadata(path).map_err(BookError::invalid)?;
        Ok(BookMeta { title: m.title, author: m.author, format: Format::Mobi })
      }
      Format::Txt => txt::parse_metadata(path),
    }
  }
}
