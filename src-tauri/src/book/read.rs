//! 给阅读器取元数据、目录和章节。文件来自这本书上的上传记录。

use std::path::{Path, PathBuf};

use serde::Serialize;

use super::OpenedBook;
use super::sanitize::sanitize_html;
use super::session;
use super::types::{BookError, Format};
use crate::db::error::DbError;
use crate::db::state::DbState;
use crate::owner::Owner;

/// 导入前读到的书名和作者
///
/// # 字段
/// - `title`: 书名。解析不到时可能为空，调用方再用文件名
/// - `author`: 作者
/// - `format`: `epub`、`mobi` 或 `txt`
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookPeek {
  /// 书名
  pub title: String,
  /// 作者
  pub author: String,
  /// 格式名
  pub format: String,
}

/// 目录里的一章
///
/// # 字段
/// - `index`: 从 0 开始
/// - `title`: 章节标题
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlineItem {
  /// 章节下标
  pub index: usize,
  /// 章节标题
  pub title: String,
}

/// 一章正文
///
/// # 字段
/// - `index`: 从 0 开始
/// - `title`: 章节标题
/// - `content`: EPUB、MOBI 为清理后的 HTML，TXT 为纯文本
/// - `html`: 为真时按 HTML 渲染
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterBody {
  /// 章节下标
  pub index: usize,
  /// 章节标题
  pub title: String,
  /// 章节正文
  pub content: String,
  /// 正文是否为 HTML
  pub html: bool,
}

/// 阅读一本书时的失败
///
/// # 变体
/// - `NoFile`: 这本书没有可阅读的文件
/// - `Encrypted`: 文件已加密，阅读器打不开
/// - `Book`: 解析失败
/// - `Locked`: 数据库未解锁
/// - `Internal`: 查询失败
#[derive(Debug, thiserror::Error)]
pub enum ReadError {
  /// 这本书没有可阅读的文件
  #[error("这本书还没有正文")]
  NoFile,
  /// 文件已加密
  #[error("这本书已加密，暂时不能阅读")]
  Encrypted,
  /// 解析失败
  #[error(transparent)]
  Book(#[from] BookError),
  /// 数据库未解锁
  #[error("数据库未解锁")]
  Locked,
  /// 查询失败
  #[error("操作失败")]
  Internal,
}

impl From<DbError> for ReadError {
  /// 把数据库错误转成阅读错误
  ///
  /// # 参数
  /// - `err`: 数据库错误
  ///
  /// # 返回值
  /// 未解锁时返回未解锁，其余返回内部错误
  ///
  /// # Panics
  /// 不会主动 panic
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => ReadError::Locked,
      _ => ReadError::Internal,
    }
  }
}

impl Serialize for ReadError {
  /// 把阅读错误序列化为错误文本
  ///
  /// # 参数
  /// - `serializer`: 序列化器
  ///
  /// # 返回值
  /// 返回序列化器写入后的结果
  ///
  /// # 错误
  /// 序列化失败时返回序列化器错误
  ///
  /// # Panics
  /// 不会主动 panic
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}

/// 从本地路径读出书名、作者和格式
///
/// # 参数
/// - `path`: 用户选中的文件
///
/// # 返回值
/// 成功返回元数据
///
/// # 错误
/// 认不出格式或解析失败时返回阅读错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let meta = peek(Path::new("book.txt"))?;
/// ```
pub fn peek(path: &Path) -> Result<BookPeek, ReadError> {
  let meta = OpenedBook::parse_metadata(path)?;
  Ok(BookPeek { title: meta.title, author: meta.author, format: meta.format.as_str().to_string() })
}

/// 列出这本书的目录
///
/// # 参数
/// - `db`: 数据库状态
/// - `book_id`: 书籍标识
///
/// # 返回值
/// 返回从 0 编号的章节标题
///
/// # 错误
/// 没有文件、文件加密或解析失败时返回阅读错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let outline = outline(db, book_id)?;
/// ```
pub fn outline(db: &DbState, book_id: &str) -> Result<Vec<OutlineItem>, ReadError> {
  let path = book_file(db, book_id)?;
  let outline = session::with_book(&path, |book| Ok(book.outline()))?;
  Ok(outline.into_iter().map(|item| OutlineItem { index: item.index, title: item.title }).collect())
}

/// 取出一章正文
///
/// EPUB 和 MOBI 会先去掉脚本。TXT 保持纯文本。
///
/// # 参数
/// - `db`: 数据库状态
/// - `book_id`: 书籍标识
/// - `index`: 从 0 开始的章节下标
///
/// # 返回值
/// 返回这一章的标题和正文
///
/// # 错误
/// 没有文件、下标越界或解析失败时返回阅读错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let chapter = chapter(db, book_id, 0)?;
/// ```
pub fn chapter(db: &DbState, book_id: &str, index: usize) -> Result<ChapterBody, ReadError> {
  let path = book_file(db, book_id)?;
  session::with_book(&path, |book| {
    let mut chapter = book.chapter(index)?;
    let html = book.metadata().format != Format::Txt;
    if html {
      chapter.content = sanitize_html(&chapter.content);
    }
    Ok(ChapterBody { index: chapter.index, title: chapter.title, content: chapter.content, html })
  })
  .map_err(ReadError::from)
}

/// 找到这本书上最新的未加密书籍文件
///
/// # 参数
/// - `db`: 数据库状态
/// - `book_id`: 书籍标识
///
/// # 返回值
/// 返回可打开的绝对路径
///
/// # 错误
/// 没有文件时返回没有正文；只有加密文件时返回已加密；查询失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
fn book_file(db: &DbState, book_id: &str) -> Result<PathBuf, ReadError> {
  let rows =
    db.with_conn(|conn| crate::upload::list_for(conn, Owner::Book.as_str(), book_id).map_err(|_| ReadError::Internal))?;
  let books: Vec<_> = rows.into_iter().filter(|row| row.kind == "book").collect();
  if let Some(file) = books.iter().rev().find(|row| !row.encrypted) {
    return Ok(crate::upload::resolve(db, &file.rel_path));
  }
  if books.iter().any(|row| row.encrypted) {
    return Err(ReadError::Encrypted);
  }
  Err(ReadError::NoFile)
}
