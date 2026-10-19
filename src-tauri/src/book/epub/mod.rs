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
  /// 打开 EPUB 并缓存包信息
  ///
  /// 解析压缩包里的 container、OPF 和导航，书名与作者取自包元数据。
  ///
  /// # 参数
  /// - `path`: EPUB 文件路径
  ///
  /// # 返回值
  /// 成功返回缓存了路径和包信息的 `EpubBook`
  ///
  /// # 错误
  /// 解析包失败时返回 `BookError::Invalid`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = EpubBook::open(Path::new("book.epub")).unwrap();
  /// assert_eq!(book.metadata().format, Format::Epub);
  /// ```
  pub fn open(path: &Path) -> Result<Self, BookError> {
    let package = parse_package(path).map_err(BookError::invalid)?;
    let meta =
      BookMeta { title: package.meta.title.clone(), author: package.meta.author.clone(), format: Format::Epub };
    Ok(Self { path: path.to_path_buf(), meta, package })
  }

  /// 返回已缓存的书籍元数据
  ///
  /// # 返回值
  /// 打开时从 OPF 取出的元数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = EpubBook::open(Path::new("book.epub")).unwrap();
  /// let _meta = book.metadata();
  /// ```
  pub fn metadata(&self) -> &BookMeta {
    &self.meta
  }

  /// 返回 spine 目录标题
  ///
  /// 按 spine 顺序生成目录。标题为空时写成 `第 {n} 章`，序号从 1 开始。spine 为空时返回一章「正文」。
  ///
  /// # 返回值
  /// 不含正文的目录项
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = EpubBook::open(Path::new("book.epub")).unwrap();
  /// let outline = book.outline();
  /// assert_eq!(outline[0].index, 0);
  /// ```
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

  /// 按 spine 索引读取一章
  ///
  /// spine 为空时，索引 0 返回空正文的「正文」，其他索引越界。否则组装该条目的标题和正文。
  ///
  /// # 参数
  /// - `index`: 从 0 开始的 spine 下标
  ///
  /// # 返回值
  /// 成功返回该章标题和正文
  ///
  /// # 错误
  /// 下标越界时返回 `BookError::ChapterOutOfRange`；组装章节失败时返回 `BookError::Invalid`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = EpubBook::open(Path::new("book.epub")).unwrap();
  /// let chapter = book.chapter(0).unwrap();
  /// assert_eq!(chapter.index, 0);
  /// ```
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

  /// 读取全部 spine 章节
  ///
  /// spine 为空时返回一章空的「正文」，否则按索引逐章读取。
  ///
  /// # 返回值
  /// 成功返回元数据和全部章节
  ///
  /// # 错误
  /// 任一章读取失败时返回该章的错误
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = EpubBook::open(Path::new("book.epub")).unwrap();
  /// let (_meta, chapters) = book.content().unwrap();
  /// assert!(!chapters.is_empty());
  /// ```
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

/// 解析 EPUB 的标题和作者
///
/// 打开压缩包并解析 OPF 后，只返回包里的轻量元数据。
///
/// # 参数
/// - `path`: EPUB 文件路径
///
/// # 返回值
/// 成功返回标题和作者
///
/// # 错误
/// 打开或解析包失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let meta = parse_metadata(Path::new("book.epub")).unwrap();
/// assert!(!meta.title.is_empty());
/// ```
pub(super) fn parse_metadata(path: &Path) -> Result<meta::EpubMeta, error::EpubError> {
  let package = parse_package(path)?;
  Ok(package.meta)
}
