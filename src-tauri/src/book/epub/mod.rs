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
use std::sync::OnceLock;

use crate::book::blank::chapter_is_blank;
use crate::book::types::{BookError, BookMeta, Chapter, ChapterRef, Format};

use content::{assemble_epub_spine_entry, extract_body_or_raw};
use package::{open_zip, parse_package, read_zip_text_from_archive};

/// 已打开的 EPUB：缓存 OPF/spine，按章懒加载正文
///
/// # 字段
/// - `path`: EPUB 文件路径，读章节时再打开
/// - `meta`: 书名和作者，格式为 [`Format::Epub`]
/// - `package`: 缓存的 OPF 包，目录来自其中的 spine
/// - `readable`: 去掉空白 spine 后保留的下标。第一次列目录或读章节时填上
///
/// # 示例
/// 见 [`open`]
pub struct EpubBook {
  /// EPUB 文件路径，读章节时再打开
  path: PathBuf,
  /// 书名和作者，格式为 [`Format::Epub`]
  meta: BookMeta,
  /// 缓存的 OPF 包，目录来自其中的 spine
  package: package::ParsedPackage,
  /// 去掉空白 spine 后保留的下标。第一次列目录或读章节时填上
  readable: OnceLock<Vec<usize>>,
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
    Ok(Self { path: path.to_path_buf(), meta, package, readable: OnceLock::new() })
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
  /// 按 spine 顺序生成目录，丢掉正文为空白的条目，剩下的从 0 编号。标题为空时写成 `第 {n} 章`，序号从 1 开始。全部为空时返回空目录。
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
    let kept = self.readable_spine().to_vec();
    kept
      .iter()
      .enumerate()
      .map(|(index, spine_index)| {
        let entry = &self.package.spine[*spine_index];
        ChapterRef {
          index,
          title: if entry.title.is_empty() { format!("第 {} 章", index + 1) } else { entry.title.clone() },
        }
      })
      .collect()
  }

  /// 按去掉空白章节后的索引读取一章
  ///
  /// 下标对应 [`outline`] 的顺序，不是原始 spine 下标。
  ///
  /// # 参数
  /// - `index`: 从 0 开始、已去掉空白章节的下标
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
    let spine_index = *self.readable_spine().get(index).ok_or(BookError::ChapterOutOfRange)?;
    let entry = self.package.spine.get(spine_index).ok_or(BookError::ChapterOutOfRange)?;
    let ch = assemble_epub_spine_entry(&self.path, entry, index).map_err(BookError::invalid)?;
    Ok(Chapter { index, title: ch.title, content: ch.content })
  }

  /// 列出正文不为空白的 spine 下标
  ///
  /// 第一次调用时打开压缩包，逐条读出正文并跳过空白。结果缓存在这本书上。打不开压缩包时返回空列表。
  ///
  /// # 返回值
  /// 保留下来的 spine 下标
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = EpubBook::open(Path::new("book.epub")).unwrap();
  /// let _kept = book.readable_spine();
  /// ```
  fn readable_spine(&self) -> &[usize] {
    self.readable.get_or_init(|| self.scan_readable_spine())
  }

  /// 扫描 spine，收集正文不为空白的下标
  ///
  /// 压缩包只打开一次。读不到的条目按空白丢掉。
  ///
  /// # 返回值
  /// 保留下来的 spine 下标
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = EpubBook::open(Path::new("book.epub")).unwrap();
  /// let _kept = book.scan_readable_spine();
  /// ```
  fn scan_readable_spine(&self) -> Vec<usize> {
    let mut zip = match open_zip(&self.path) {
      Ok(zip) => zip,
      Err(_) => return Vec::new(),
    };
    self
      .package
      .spine
      .iter()
      .enumerate()
      .filter_map(|(index, entry)| {
        let raw = read_zip_text_from_archive(&mut zip, &entry.href).unwrap_or_default();
        let body = extract_body_or_raw(&raw);
        if chapter_is_blank(&body) { None } else { Some(index) }
      })
      .collect()
  }

  /// 读取全部有正文的 spine 章节
  ///
  /// 空白章节不包含在结果里。
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
    let count = self.readable_spine().len();
    let mut chapters = Vec::with_capacity(count);
    for index in 0..count {
      chapters.push(self.chapter(index)?);
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
