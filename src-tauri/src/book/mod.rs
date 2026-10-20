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
///
/// # 变体
/// - `Epub`: 已打开的 [`EpubBook`]
/// - `Mobi`: 已打开的 [`MobiBook`]
/// - `Txt`: 已打开的 [`TxtBook`]
pub enum OpenedBook {
  /// 已打开的 [`EpubBook`]
  Epub(EpubBook),
  /// 已打开的 [`MobiBook`]
  Mobi(MobiBook),
  /// 已打开的 [`TxtBook`]
  Txt(TxtBook),
}

impl OpenedBook {
  /// 按路径探测格式并打开电子书
  ///
  /// 探测失败时返回不支持的格式。探测成功后交给已知格式的打开逻辑。
  ///
  /// # 参数
  /// - `path`: 书籍文件路径
  ///
  /// # 返回值
  /// 成功返回对应格式的已打开书籍
  ///
  /// # 错误
  /// 认不出格式时返回 `BookError::Unsupported`；打开过程中的错误原样返回
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = OpenedBook::open(Path::new("book.epub")).unwrap();
  /// let _meta = book.metadata();
  /// ```
  pub fn open(path: &Path) -> Result<Self, BookError> {
    let format = detect_format(path).ok_or(BookError::Unsupported)?;
    Self::open_with_format(path, format)
  }

  /// 用已知格式打开电子书
  ///
  /// 路径不存在时返回未找到。存在时按 `format` 分别打开 EPUB、MOBI 或 TXT。
  ///
  /// # 参数
  /// - `path`: 书籍文件路径
  /// - `format`: 已经确定的格式
  ///
  /// # 返回值
  /// 成功返回对应格式的已打开书籍
  ///
  /// # 错误
  /// 文件不存在时返回 `BookError::NotFound`；各格式打开失败时返回其转换后的错误
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = OpenedBook::open_with_format(Path::new("book.txt"), Format::Txt).unwrap();
  /// assert_eq!(book.metadata().format, Format::Txt);
  /// ```
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

  /// 返回当前书籍的元数据
  ///
  /// # 返回值
  /// 内部 EPUB、MOBI 或 TXT 书籍的元数据引用
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = OpenedBook::open(Path::new("book.txt")).unwrap();
  /// let _meta = book.metadata();
  /// ```
  pub fn metadata(&self) -> &BookMeta {
    match self {
      OpenedBook::Epub(b) => b.metadata(),
      OpenedBook::Mobi(b) => b.metadata(),
      OpenedBook::Txt(b) => b.metadata(),
    }
  }

  /// 返回当前书籍的章节目录
  ///
  /// # 返回值
  /// 转发给内部书籍得到的目录项
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = OpenedBook::open(Path::new("book.txt")).unwrap();
  /// let _outline = book.outline();
  /// ```
  pub fn outline(&self) -> Vec<ChapterRef> {
    match self {
      OpenedBook::Epub(b) => b.outline(),
      OpenedBook::Mobi(b) => b.outline(),
      OpenedBook::Txt(b) => b.outline(),
    }
  }

  /// 按索引取一章正文
  ///
  /// # 参数
  /// - `index`: 从 0 开始的章节下标
  ///
  /// # 返回值
  /// 成功返回内部书籍的该章
  ///
  /// # 错误
  /// 内部书籍取章失败时返回其错误
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = OpenedBook::open(Path::new("book.txt")).unwrap();
  /// let chapter = book.chapter(0).unwrap();
  /// assert_eq!(chapter.index, 0);
  /// ```
  pub fn chapter(&self, index: usize) -> Result<Chapter, BookError> {
    match self {
      OpenedBook::Epub(b) => b.chapter(index),
      OpenedBook::Mobi(b) => b.chapter(index),
      OpenedBook::Txt(b) => b.chapter(index),
    }
  }

  /// 取出全书元数据和全部章节
  ///
  /// # 返回值
  /// 成功返回内部书籍交出的元数据和章节
  ///
  /// # 错误
  /// 内部书籍取全文失败时返回其错误
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = OpenedBook::open(Path::new("book.txt")).unwrap();
  /// let (_meta, chapters) = book.content().unwrap();
  /// assert!(!chapters.is_empty());
  /// ```
  pub fn content(self) -> Result<(BookMeta, Vec<Chapter>), BookError> {
    match self {
      OpenedBook::Epub(b) => b.content(),
      OpenedBook::Mobi(b) => b.content(),
      OpenedBook::Txt(b) => b.content(),
    }
  }

  /// 解析路径对应书籍的元数据
  ///
  /// 先探测格式。EPUB 取包内标题和作者，MOBI 取解析器标题和作者，TXT 走文本片头提取。
  ///
  /// # 参数
  /// - `path`: 书籍文件路径
  ///
  /// # 返回值
  /// 成功返回统一的 `BookMeta`
  ///
  /// # 错误
  /// 认不出格式时返回 `BookError::Unsupported`；各格式解析失败时返回 `BookError::Invalid` 或 TXT 的读取错误
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let meta = OpenedBook::parse_metadata(Path::new("book.epub")).unwrap();
  /// assert_eq!(meta.format, Format::Epub);
  /// ```
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
