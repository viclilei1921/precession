use tauri::State;

use super::dto::{Book, BookInput, BookNote, BookNoteInput};
use super::error::LibraryError;
use super::service;
use crate::db::state::DbState;

/// 列出书籍
///
/// # 参数
/// - `state`: 数据库状态
///
/// # 返回值
/// 返回书籍列表
///
/// # 错误
/// 查询失败时返回书库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = book_list(state).await?;
/// ```
#[tauri::command]
pub async fn book_list(state: State<'_, DbState>) -> Result<Vec<Book>, LibraryError> {
  service::book_list(&state)
}

/// 按标识读取书籍
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 书籍标识
///
/// # 返回值
/// 返回对应的书籍
///
/// # 错误
/// 读取失败时返回书库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = book_get(state, id).await?;
/// ```
#[tauri::command]
pub async fn book_get(state: State<'_, DbState>, id: String) -> Result<Book, LibraryError> {
  service::book_get(&state, id.as_str())
}

/// 创建书籍
///
/// # 参数
/// - `state`: 数据库状态
/// - `input`: 书籍输入
///
/// # 返回值
/// 返回新建的书籍
///
/// # 错误
/// 创建失败时返回书库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = book_create(state, input).await?;
/// ```
#[tauri::command]
pub async fn book_create(state: State<'_, DbState>, input: BookInput) -> Result<Book, LibraryError> {
  service::book_create(&state, input)
}

/// 更新书籍
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 书籍标识
/// - `input`: 书籍输入
///
/// # 返回值
/// 返回更新后的书籍
///
/// # 错误
/// 更新失败时返回书库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = book_update(state, id, input).await?;
/// ```
#[tauri::command]
pub async fn book_update(state: State<'_, DbState>, id: String, input: BookInput) -> Result<Book, LibraryError> {
  service::book_update(&state, id.as_str(), input)
}

/// 删除书籍
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 书籍标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 删除失败时返回书库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// book_delete(state, id).await?;
/// ```
#[tauri::command]
pub async fn book_delete(state: State<'_, DbState>, id: String) -> Result<(), LibraryError> {
  service::book_delete(&state, id.as_str())
}

/// 按书籍列出书摘和笔记
///
/// # 参数
/// - `state`: 数据库状态
/// - `book_id`: 书籍标识
///
/// # 返回值
/// 返回该书的书摘和笔记
///
/// # 错误
/// 查询失败时返回书库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = book_note_list(state, book_id).await?;
/// ```
#[tauri::command]
pub async fn book_note_list(state: State<'_, DbState>, book_id: String) -> Result<Vec<BookNote>, LibraryError> {
  service::note_list(&state, book_id.as_str())
}

/// 按标识读取书摘或笔记
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 书摘或笔记标识
///
/// # 返回值
/// 返回对应的书摘或笔记
///
/// # 错误
/// 读取失败时返回书库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = book_note_get(state, id).await?;
/// ```
#[tauri::command]
pub async fn book_note_get(state: State<'_, DbState>, id: String) -> Result<BookNote, LibraryError> {
  service::note_get(&state, id.as_str())
}

/// 创建书摘或笔记
///
/// # 参数
/// - `state`: 数据库状态
/// - `input`: 书摘或笔记输入
///
/// # 返回值
/// 返回新建的书摘或笔记
///
/// # 错误
/// 创建失败时返回书库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = book_note_create(state, input).await?;
/// ```
#[tauri::command]
pub async fn book_note_create(state: State<'_, DbState>, input: BookNoteInput) -> Result<BookNote, LibraryError> {
  service::note_create(&state, input)
}

/// 更新书摘或笔记
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 书摘或笔记标识
/// - `input`: 书摘或笔记输入
///
/// # 返回值
/// 返回更新后的书摘或笔记
///
/// # 错误
/// 更新失败时返回书库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = book_note_update(state, id, input).await?;
/// ```
#[tauri::command]
pub async fn book_note_update(
  state: State<'_, DbState>,
  id: String,
  input: BookNoteInput,
) -> Result<BookNote, LibraryError> {
  service::note_update(&state, id.as_str(), input)
}

/// 删除书摘或笔记
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 书摘或笔记标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 删除失败时返回书库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// book_note_delete(state, id).await?;
/// ```
#[tauri::command]
pub async fn book_note_delete(state: State<'_, DbState>, id: String) -> Result<(), LibraryError> {
  service::note_delete(&state, id.as_str())
}
