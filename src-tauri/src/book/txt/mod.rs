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
  /// 打开 TXT 并按章节切分
  ///
  /// 用文件名主干作书名回退，读取全文后切出元数据和章节。
  ///
  /// # 参数
  /// - `path`: TXT 文件路径
  ///
  /// # 返回值
  /// 成功返回已缓存正文的 `TxtBook`
  ///
  /// # 错误
  /// 读文件或解码失败时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = TxtBook::open(Path::new("book.txt")).unwrap();
  /// assert_eq!(book.metadata().format, crate::book::types::Format::Txt);
  /// ```
  pub fn open(path: &Path) -> Result<Self, BookError> {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("未知标题");
    let text = parse_txt_raw(path)?;
    let (meta, chapters) = split_txt_book(&text, stem);
    Ok(Self { meta, chapters })
  }

  /// 返回已缓存的书籍元数据
  ///
  /// # 返回值
  /// 打开时切分得到的元数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = TxtBook::open(Path::new("book.txt")).unwrap();
  /// let _meta = book.metadata();
  /// ```
  pub fn metadata(&self) -> &BookMeta {
    &self.meta
  }

  /// 返回不含正文的章节目录
  ///
  /// # 返回值
  /// 每章的索引和标题
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = TxtBook::open(Path::new("book.txt")).unwrap();
  /// let outline = book.outline();
  /// assert_eq!(outline[0].index, 0);
  /// ```
  pub fn outline(&self) -> Vec<ChapterRef> {
    self.chapters.iter().map(|c| ChapterRef { index: c.index, title: c.title.clone() }).collect()
  }

  /// 按索引克隆一章
  ///
  /// # 参数
  /// - `index`: 从 0 开始的章节下标
  ///
  /// # 返回值
  /// 成功返回该章的索引、标题和正文
  ///
  /// # 错误
  /// 下标超出章节列表时返回 `BookError::ChapterOutOfRange`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = TxtBook::open(Path::new("book.txt")).unwrap();
  /// let chapter = book.chapter(0).unwrap();
  /// assert_eq!(chapter.index, 0);
  /// ```
  pub fn chapter(&self, index: usize) -> Result<Chapter, BookError> {
    self.chapters.get(index).cloned().ok_or(BookError::ChapterOutOfRange)
  }

  /// 交出缓存的元数据和全部章节
  ///
  /// # 返回值
  /// 始终返回 `Ok`，内容是打开时切好的元数据和章节
  ///
  /// # 错误
  /// 当前实现不会返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = TxtBook::open(Path::new("book.txt")).unwrap();
  /// let (_meta, chapters) = book.content().unwrap();
  /// assert!(!chapters.is_empty());
  /// ```
  pub fn content(self) -> Result<(BookMeta, Vec<Chapter>), BookError> {
    Ok((self.meta, self.chapters))
  }
}

/// 从 TXT 片头提取书名和作者
///
/// 解码全文后只查看前 8192 个字符。逐行提取书名和作者，遇到章节标题或行号超过 12 就停止。书名仍为空时用文件名主干。
///
/// # 参数
/// - `path`: TXT 文件路径
///
/// # 返回值
/// 成功返回格式为 TXT 的元数据
///
/// # 错误
/// 读文件或解码失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let meta = parse_metadata(Path::new("book.txt")).unwrap();
/// assert_eq!(meta.format, crate::book::types::Format::Txt);
/// ```
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
