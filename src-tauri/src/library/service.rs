//! 书库。书摘和笔记写入 `book_note`。

use super::constants::{STATUS_FINISHED, STATUS_READING, STATUS_WANT};
use super::dto::{Book, BookInput, BookNote, BookNoteInput};
use super::error::LibraryError;
use super::repository;
use crate::db::state::DbState;

const STATUSES: &[&str] = &[STATUS_WANT, STATUS_READING, STATUS_FINISHED];

/// 列出书籍
///
/// # 参数
/// - `db`: 数据库状态
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
/// let rows = book_list(db)?;
/// ```
pub fn book_list(db: &DbState) -> Result<Vec<Book>, LibraryError> {
  db.with_conn(repository::book_list)
}

/// 按标识读取书籍
///
/// # 参数
/// - `db`: 数据库状态
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
/// let row = book_get(db, id)?;
/// ```
pub fn book_get(db: &DbState, id: &str) -> Result<Book, LibraryError> {
  db.with_conn(|conn| repository::book_get(conn, id))
}

/// 校验输入后创建书籍
///
/// # 参数
/// - `db`: 数据库状态
/// - `input`: 书籍输入
///
/// # 返回值
/// 返回新建的书籍
///
/// # 错误
/// 输入不合法或写入失败时返回书库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = book_create(db, input)?;
/// ```
pub fn book_create(db: &DbState, input: BookInput) -> Result<Book, LibraryError> {
  let input = prepare_book(input)?;
  db.with_conn(|conn| repository::book_create(conn, &input))
}

/// 校验输入后更新书籍
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 书籍标识
/// - `input`: 书籍输入
///
/// # 返回值
/// 返回更新后的书籍
///
/// # 错误
/// 输入不合法或更新失败时返回书库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = book_update(db, id, input)?;
/// ```
pub fn book_update(db: &DbState, id: &str, input: BookInput) -> Result<Book, LibraryError> {
  let input = prepare_book(input)?;
  db.with_conn(|conn| repository::book_update(conn, id, &input))
}

/// 按标识删除书籍，再在后台清理它留下的上传文件
///
/// # 参数
/// - `db`: 数据库状态
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
/// book_delete(db, id)?;
/// ```
pub fn book_delete(db: &DbState, id: &str) -> Result<(), LibraryError> {
  db.with_conn(|conn| repository::book_delete(conn, id))?;
  crate::upload::sweep_later(db);
  Ok(())
}

/// 按书籍列出书摘和笔记
///
/// # 参数
/// - `db`: 数据库状态
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
/// let rows = note_list(db, book_id)?;
/// ```
pub fn note_list(db: &DbState, book_id: &str) -> Result<Vec<BookNote>, LibraryError> {
  db.with_conn(|conn| repository::note_list(conn, book_id))
}

/// 按标识读取书摘或笔记
///
/// # 参数
/// - `db`: 数据库状态
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
/// let row = note_get(db, id)?;
/// ```
pub fn note_get(db: &DbState, id: &str) -> Result<BookNote, LibraryError> {
  db.with_conn(|conn| repository::note_get(conn, id))
}

/// 校验输入后创建书摘或笔记
///
/// # 参数
/// - `db`: 数据库状态
/// - `input`: 书摘或笔记输入
///
/// # 返回值
/// 返回新建的书摘或笔记
///
/// # 错误
/// 输入不合法或写入失败时返回书库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = note_create(db, input)?;
/// ```
pub fn note_create(db: &DbState, input: BookNoteInput) -> Result<BookNote, LibraryError> {
  let input = prepare_note(input)?;
  db.with_conn(|conn| repository::note_create(conn, &input))
}

/// 校验输入后更新书摘或笔记
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 书摘或笔记标识
/// - `input`: 书摘或笔记输入
///
/// # 返回值
/// 返回更新后的书摘或笔记
///
/// # 错误
/// 输入不合法或更新失败时返回书库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = note_update(db, id, input)?;
/// ```
pub fn note_update(db: &DbState, id: &str, input: BookNoteInput) -> Result<BookNote, LibraryError> {
  let input = prepare_note(input)?;
  db.with_conn(|conn| repository::note_update(conn, id, &input))
}

/// 按标识删除书摘或笔记，再在后台清理它留下的上传文件
///
/// # 参数
/// - `db`: 数据库状态
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
/// note_delete(db, id)?;
/// ```
pub fn note_delete(db: &DbState, id: &str) -> Result<(), LibraryError> {
  db.with_conn(|conn| repository::note_delete(conn, id))?;
  crate::upload::sweep_later(db);
  Ok(())
}

/// 整理并校验书籍输入
///
/// # 参数
/// - `input`: 待写入的书籍
///
/// # 返回值
/// 返回去掉标题、作者、封面路径和状态首尾空白后的输入
///
/// # 错误
/// 标题为空时返回标题不能为空；状态不受支持时返回状态不正确；进度不在 0 到 1 之间时返回进度不正确
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let input = prepare_book(raw);
/// ```
fn prepare_book(mut input: BookInput) -> Result<BookInput, LibraryError> {
  input.title = input.title.trim().to_string();
  input.author = input.author.trim().to_string();
  input.cover_path = input.cover_path.trim().to_string();
  input.status = input.status.trim().to_string();
  if input.title.is_empty() {
    return Err(LibraryError::TitleEmpty);
  }
  if !STATUSES.contains(&input.status.as_str()) {
    return Err(LibraryError::StatusInvalid);
  }
  if !(0.0..=1.0).contains(&input.progress) {
    return Err(LibraryError::ProgressInvalid);
  }
  Ok(input)
}

/// 整理并校验书摘或笔记输入
///
/// # 参数
/// - `input`: 待写入的书摘或笔记
///
/// # 返回值
/// 返回去掉书籍标识、类型、标题、正文、章节和位置首尾空白后的输入
///
/// # 错误
/// 标题为空时返回标题不能为空；类型不受支持时返回类型不正确
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let input = prepare_note(raw);
/// ```
fn prepare_note(mut input: BookNoteInput) -> Result<BookNoteInput, LibraryError> {
  input.book_id = input.book_id.trim().to_string();
  input.kind = input.kind.trim().to_string();
  input.title = input.title.trim().to_string();
  input.body = input.body.trim().to_string();
  input.chapter = input.chapter.trim().to_string();
  input.location = input.location.trim().to_string();
  if input.title.is_empty() {
    return Err(LibraryError::TitleEmpty);
  }
  if !repository::allowed_kind(&input.kind) {
    return Err(LibraryError::KindInvalid);
  }
  Ok(input)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::library::constants::KIND_EXCERPT;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = DbState::new(dir.path().to_path_buf()).with_migrators(vec![
      crate::member::migrate,
      crate::tag::migrate,
      crate::place::migrate,
      crate::upload::migrate,
      crate::journal::migrate,
      crate::library::migrate,
    ]);
    db.create("test-password-123", "test-password-123").expect("create db");
    (dir, db)
  }

  fn book() -> BookInput {
    BookInput {
      title: "置身事内".into(),
      author: "兰小欢".into(),
      cover_path: String::new(),
      status: STATUS_READING.into(),
      progress: 0.62,
      rating: None,
      started_at: Some(1),
      finished_at: None,
    }
  }

  #[test]
  fn book_note_and_lock() {
    let (_dir, db) = setup();
    let mut bad = book();
    bad.progress = 1.2;
    assert!(matches!(book_create(&db, bad), Err(LibraryError::ProgressInvalid)));

    let saved = book_create(&db, book()).expect("book");
    let note = note_create(
      &db,
      BookNoteInput {
        book_id: saved.id.clone(),
        kind: KIND_EXCERPT.into(),
        occurred_at: 20,
        title: "事权与财权".into(),
        body: "划分与再平衡".into(),
        chapter: "第 4 章".into(),
        location: "P301".into(),
        locked: false,
        highlight: false,
        member_ids: Vec::new(),
        tag_ids: Vec::new(),
        place_ids: Vec::new(),
      },
    )
    .expect("note");
    assert_eq!(note_list(&db, &saved.id).expect("notes").len(), 1);
    assert_eq!(note.location, "P301");

    db.lock().expect("lock");
    assert!(matches!(book_list(&db), Err(LibraryError::Locked)));
  }
}
