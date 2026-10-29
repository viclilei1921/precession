use std::path::PathBuf;

use tauri::State;

use super::read::{self, BookPeek, ChapterBody, OutlineItem, ReadError};
use crate::db::state::DbState;

/// 从用户选中的文件读出书名、作者和格式
///
/// # 参数
/// - `path`: 本地文件路径
///
/// # 返回值
/// 返回书名、作者和格式
///
/// # 错误
/// 认不出格式或解析失败时返回阅读错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let meta = book_peek(path)?;
/// ```
#[tauri::command]
pub fn book_peek(path: String) -> Result<BookPeek, ReadError> {
  read::peek(PathBuf::from(path).as_path())
}

/// 列出一本书的目录
///
/// # 参数
/// - `state`: 数据库状态
/// - `book_id`: 书籍标识
///
/// # 返回值
/// 返回章节标题
///
/// # 错误
/// 没有正文或解析失败时返回阅读错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let outline = book_outline(state, book_id)?;
/// ```
#[tauri::command]
pub fn book_outline(state: State<'_, DbState>, book_id: String) -> Result<Vec<OutlineItem>, ReadError> {
  read::outline(&state, book_id.as_str())
}

/// 读取一章正文
///
/// # 参数
/// - `state`: 数据库状态
/// - `book_id`: 书籍标识
/// - `index`: 从 0 开始的章节下标
///
/// # 返回值
/// 返回这一章
///
/// # 错误
/// 没有正文、下标越界或解析失败时返回阅读错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let chapter = book_chapter(state, book_id, index)?;
/// ```
#[tauri::command]
pub fn book_chapter(state: State<'_, DbState>, book_id: String, index: usize) -> Result<ChapterBody, ReadError> {
  read::chapter(&state, book_id.as_str(), index)
}
