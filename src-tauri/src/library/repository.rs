//! 书和书摘。书摘是独立的一条记录，不再挂在统一记录表上。

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::{BOOK_TABLE, KIND_EXCERPT, KIND_NOTE, NOTE_TABLE, SCHEMA_VERSION, SCHEMA_VERSION_KEY};
use super::dto::{Book, BookInput, BookNote, BookNoteInput};
use super::error::LibraryError;
use crate::owner::Owner;
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

/// 记录数据库错误并返回内部错误
///
/// # 参数
/// - `context`: 出错位置说明
/// - `err`: 数据库错误
///
/// # 返回值
/// 始终返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let err = db_fail("book list", rusqlite_err);
/// ```
fn db_fail(context: &str, err: rusqlite::Error) -> LibraryError {
  tauri_plugin_log::log::error!("{context}: {err}");
  LibraryError::Internal
}

const BOOK_COLUMNS: &str =
  "id, title, author, cover_path, status, progress, rating, started_at, finished_at, created_at, updated_at";
const NOTE_COLUMNS: &str =
  "id, book_id, kind, occurred_at, title, body, chapter, location, locked, highlight, created_at, updated_at";

/// 把查询行映射为书籍
///
/// # 参数
/// - `row`: 查询结果行
///
/// # 返回值
/// 返回书籍
///
/// # 错误
/// 读取列失败时返回数据库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let book = map_book(row)?;
/// ```
fn map_book(row: &rusqlite::Row<'_>) -> rusqlite::Result<Book> {
  Ok(Book {
    id: row.get(0)?,
    title: row.get(1)?,
    author: row.get(2)?,
    cover_path: row.get(3)?,
    status: row.get(4)?,
    progress: row.get(5)?,
    rating: row.get(6)?,
    started_at: row.get(7)?,
    finished_at: row.get(8)?,
    created_at: row.get(9)?,
    updated_at: row.get(10)?,
  })
}

/// 把查询行映射为书摘或笔记
///
/// # 参数
/// - `row`: 查询结果行
///
/// # 返回值
/// 返回成员、标签和地点引用均为空的书摘或笔记
///
/// # 错误
/// 读取列失败时返回数据库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let note = map_note(row)?;
/// ```
fn map_note(row: &rusqlite::Row<'_>) -> rusqlite::Result<BookNote> {
  Ok(BookNote {
    id: row.get(0)?,
    book_id: row.get(1)?,
    kind: row.get(2)?,
    occurred_at: row.get(3)?,
    title: row.get(4)?,
    body: row.get(5)?,
    chapter: row.get(6)?,
    location: row.get(7)?,
    locked: row.get(8)?,
    highlight: row.get(9)?,
    member_ids: Vec::new(),
    tag_ids: Vec::new(),
    place_ids: Vec::new(),
    created_at: row.get(10)?,
    updated_at: row.get(11)?,
  })
}

/// 确保书籍表、书摘表和模式版本存在
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回值
/// 表结构已就绪时返回空成功
///
/// # 错误
/// 读取版本、建表或写入版本失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// ensure_schema(conn)?;
/// ```
pub(super) fn ensure_schema(conn: &Connection) -> Result<(), LibraryError> {
  let current = crate::db::schema::read_version(conn, SCHEMA_VERSION_KEY).map_err(|e| db_fail("library schema", e))?;
  if current < 1 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS book (
           id TEXT PRIMARY KEY NOT NULL,
           title TEXT NOT NULL,
           author TEXT NOT NULL DEFAULT '',
           cover_path TEXT NOT NULL DEFAULT '',
           status TEXT NOT NULL,
           progress REAL NOT NULL DEFAULT 0,
           rating INTEGER,
           started_at INTEGER,
           finished_at INTEGER,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           deleted_at INTEGER
         );
         CREATE INDEX IF NOT EXISTS book_updated_at ON book(updated_at);
         CREATE TABLE IF NOT EXISTS book_note (
           id TEXT PRIMARY KEY NOT NULL,
           book_id TEXT NOT NULL,
           kind TEXT NOT NULL,
           occurred_at INTEGER NOT NULL,
           title TEXT NOT NULL,
           body TEXT NOT NULL DEFAULT '',
           chapter TEXT NOT NULL DEFAULT '',
           location TEXT NOT NULL DEFAULT '',
           locked INTEGER NOT NULL DEFAULT 0,
           highlight INTEGER NOT NULL DEFAULT 0,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           deleted_at INTEGER
         );
         CREATE INDEX IF NOT EXISTS book_note_book ON book_note(book_id, occurred_at);",
      )
      .map_err(|e| db_fail("library migrate v1", e))?;
  }
  if current < SCHEMA_VERSION {
    crate::db::schema::write_version(conn, SCHEMA_VERSION_KEY, SCHEMA_VERSION)
      .map_err(|e| db_fail("library schema", e))?;
  }
  Ok(())
}

/// 判断书摘或笔记类型是否受支持
///
/// # 参数
/// - `kind`: 记录类型
///
/// # 返回值
/// 类型为书摘或笔记时返回 true
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let ok = allowed_kind("excerpt");
/// ```
pub(super) fn allowed_kind(kind: &str) -> bool {
  kind == KIND_EXCERPT || kind == KIND_NOTE
}

/// 检查未删除的书籍是否存在
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 书籍标识
///
/// # 返回值
/// 存在未删除书籍时返回 true
///
/// # 错误
/// 查询失败时返回数据库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let found = book_exists(conn, id)?;
/// ```
pub(super) fn book_exists(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
  let found: Option<i64> = conn
    .query_row("SELECT 1 FROM book WHERE id = ?1 AND deleted_at IS NULL", [id], |row| {
      row.get(0)
    })
    .optional()?;
  Ok(found.is_some())
}

/// 检查未删除的书摘或笔记是否存在
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 书摘或笔记标识
///
/// # 返回值
/// 存在未删除记录时返回 true
///
/// # 错误
/// 查询失败时返回数据库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let found = note_exists(conn, id)?;
/// ```
pub(super) fn note_exists(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
  let found: Option<i64> = conn
    .query_row("SELECT 1 FROM book_note WHERE id = ?1 AND deleted_at IS NULL", [id], |row| {
      row.get(0)
    })
    .optional()?;
  Ok(found.is_some())
}

/// 按更新时间倒序列出未删除的书籍
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回值
/// 返回书籍列表
///
/// # 错误
/// 查询失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = book_list(conn)?;
/// ```
pub(super) fn book_list(conn: &Connection) -> Result<Vec<Book>, LibraryError> {
  let sql = format!("SELECT {BOOK_COLUMNS} FROM {BOOK_TABLE} WHERE deleted_at IS NULL ORDER BY updated_at DESC");
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("book list", e))?;
  let rows = stmt.query_map([], map_book).map_err(|e| db_fail("book list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("book list", e))
}

/// 按标识读取未删除的书籍
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 书籍标识
///
/// # 返回值
/// 返回对应的书籍
///
/// # 错误
/// 没有匹配记录时返回不存在；其他查询失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = book_get(conn, id)?;
/// ```
pub(super) fn book_get(conn: &Connection, id: &str) -> Result<Book, LibraryError> {
  let sql = format!("SELECT {BOOK_COLUMNS} FROM {BOOK_TABLE} WHERE id = ?1 AND deleted_at IS NULL");
  conn.query_row(&sql, [id], map_book).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => LibraryError::NotFound,
    other => db_fail("book get", other),
  })
}

/// 插入书籍并读回
///
/// # 参数
/// - `conn`: 数据库连接
/// - `input`: 书籍输入
///
/// # 返回值
/// 返回新建后重新读出的书籍
///
/// # 错误
/// 插入或读回失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = book_create(conn, input)?;
/// ```
pub(super) fn book_create(conn: &Connection, input: &BookInput) -> Result<Book, LibraryError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let sql = format!(
    "INSERT INTO {BOOK_TABLE}
       (id, title, author, cover_path, status, progress, rating, started_at, finished_at, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)"
  );
  conn
    .execute(
      &sql,
      params![
        id,
        input.title,
        input.author,
        input.cover_path,
        input.status,
        input.progress,
        input.rating,
        input.started_at,
        input.finished_at,
        now
      ],
    )
    .map_err(|e| db_fail("book create", e))?;
  book_get(conn, &id)
}

/// 更新未删除的书籍并读回
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 书籍标识
/// - `input`: 书籍输入
///
/// # 返回值
/// 返回更新后重新读出的书籍
///
/// # 错误
/// 没有匹配记录时返回不存在；更新或读回失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = book_update(conn, id, input)?;
/// ```
pub(super) fn book_update(conn: &Connection, id: &str, input: &BookInput) -> Result<Book, LibraryError> {
  let now = now_unix_ms();
  let sql = format!(
    "UPDATE {BOOK_TABLE}
     SET title = ?1, author = ?2, cover_path = ?3, status = ?4, progress = ?5, rating = ?6,
         started_at = ?7, finished_at = ?8, updated_at = ?9
     WHERE id = ?10 AND deleted_at IS NULL"
  );
  let n = conn
    .execute(
      &sql,
      params![
        input.title,
        input.author,
        input.cover_path,
        input.status,
        input.progress,
        input.rating,
        input.started_at,
        input.finished_at,
        now,
        id
      ],
    )
    .map_err(|e| db_fail("book update", e))?;
  if n == 0 {
    return Err(LibraryError::NotFound);
  }
  book_get(conn, id)
}

/// 软删除书籍及其未删除的书摘和笔记
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 书籍标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 书籍不存在、清除书摘引用失败或提交失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// book_delete(conn, id)?;
/// ```
pub(super) fn book_delete(conn: &Connection, id: &str) -> Result<(), LibraryError> {
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("book delete tx", e))?;
  let n = tx
    .execute(
      "UPDATE book SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
      params![now, id],
    )
    .map_err(|e| db_fail("book delete", e))?;
  if n == 0 {
    return Err(LibraryError::NotFound);
  }
  let ids = note_ids(&tx, id)?;
  tx.execute(
    "UPDATE book_note SET deleted_at = ?1, updated_at = ?1 WHERE book_id = ?2 AND deleted_at IS NULL",
    params![now, id],
  )
  .map_err(|e| db_fail("book notes delete", e))?;
  for note_id in ids {
    clear_note(&tx, &note_id)?;
  }
  tx.commit().map_err(|e| db_fail("book delete commit", e))?;
  Ok(())
}

/// 按书籍列出书摘和笔记
///
/// # 参数
/// - `conn`: 数据库连接
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
/// let rows = note_list(conn, book_id)?;
/// ```
pub(super) fn note_list(conn: &Connection, book_id: &str) -> Result<Vec<BookNote>, LibraryError> {
  notes(conn, None, Some(book_id), None, None)
}

/// 按发生时间范围列出书摘和笔记
///
/// # 参数
/// - `conn`: 数据库连接
/// - `from`: 起始发生时间
/// - `to`: 结束发生时间
///
/// # 返回值
/// 返回该范围内的书摘和笔记
///
/// # 错误
/// 查询失败时返回书库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list_between(conn, from, to)?;
/// ```
pub(super) fn list_between(
  conn: &Connection,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<BookNote>, LibraryError> {
  notes(conn, None, None, from, to)
}

/// 按标识读取未删除的书摘或笔记
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 书摘或笔记标识
///
/// # 返回值
/// 返回匹配的书摘或笔记
///
/// # 错误
/// 没有匹配记录时返回不存在；查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = note_get(conn, id)?;
/// ```
pub(super) fn note_get(conn: &Connection, id: &str) -> Result<BookNote, LibraryError> {
  let mut items = notes(conn, Some(id), None, None, None)?;
  items.pop().ok_or(LibraryError::NotFound)
}

/// 写入书摘或笔记并刷新所属书籍
///
/// # 参数
/// - `conn`: 数据库连接
/// - `input`: 书摘或笔记输入
///
/// # 返回值
/// 返回新建后重新读出的书摘或笔记
///
/// # 错误
/// 书籍不存在、插入失败、引用写入失败或提交失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = note_create(conn, input)?;
/// ```
pub(super) fn note_create(conn: &Connection, input: &BookNoteInput) -> Result<BookNote, LibraryError> {
  if !book_exists(conn, &input.book_id).map_err(|e| db_fail("book note", e))? {
    return Err(LibraryError::NotFound);
  }
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("book note tx", e))?;
  tx.execute(
    "INSERT INTO book_note
       (id, book_id, kind, occurred_at, title, body, chapter, location, locked, highlight, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
    params![
      id,
      input.book_id,
      input.kind,
      input.occurred_at,
      input.title,
      input.body,
      input.chapter,
      input.location,
      input.locked,
      input.highlight,
      now
    ],
  )
  .map_err(|e| db_fail("book note create", e))?;
  replace_refs(&tx, &id, input)?;
  touch_book(&tx, &input.book_id, now)?;
  tx.commit().map_err(|e| db_fail("book note commit", e))?;
  note_get(conn, &id)
}

/// 更新未删除的书摘或笔记并刷新所属书籍
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 书摘或笔记标识
/// - `input`: 书摘或笔记输入
///
/// # 返回值
/// 返回更新后重新读出的书摘或笔记
///
/// # 错误
/// 书籍或笔记不存在、更新失败或提交失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = note_update(conn, id, input)?;
/// ```
pub(super) fn note_update(conn: &Connection, id: &str, input: &BookNoteInput) -> Result<BookNote, LibraryError> {
  if !book_exists(conn, &input.book_id).map_err(|e| db_fail("book note", e))? {
    return Err(LibraryError::NotFound);
  }
  let previous = note_get(conn, id)?;
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("book note update tx", e))?;
  let n = tx
    .execute(
      "UPDATE book_note
       SET book_id = ?1, kind = ?2, occurred_at = ?3, title = ?4, body = ?5, chapter = ?6, location = ?7,
           locked = ?8, highlight = ?9, updated_at = ?10
       WHERE id = ?11 AND deleted_at IS NULL",
      params![
        input.book_id,
        input.kind,
        input.occurred_at,
        input.title,
        input.body,
        input.chapter,
        input.location,
        input.locked,
        input.highlight,
        now,
        id
      ],
    )
    .map_err(|e| db_fail("book note update", e))?;
  if n == 0 {
    return Err(LibraryError::NotFound);
  }
  replace_refs(&tx, id, input)?;
  touch_book(&tx, &input.book_id, now)?;
  if previous.book_id != input.book_id {
    touch_book(&tx, &previous.book_id, now)?;
  }
  tx.commit().map_err(|e| db_fail("book note update commit", e))?;
  note_get(conn, id)
}

/// 软删除书摘或笔记并刷新所属书籍
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 书摘或笔记标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 笔记不存在、清除引用失败或提交失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// note_delete(conn, id)?;
/// ```
pub(super) fn note_delete(conn: &Connection, id: &str) -> Result<(), LibraryError> {
  let note = note_get(conn, id)?;
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("book note delete tx", e))?;
  tx.execute(
    "UPDATE book_note SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
    params![now, id],
  )
  .map_err(|e| db_fail("book note delete", e))?;
  clear_note(&tx, id)?;
  touch_book(&tx, &note.book_id, now)?;
  tx.commit().map_err(|e| db_fail("book note delete commit", e))?;
  Ok(())
}

/// 按条件查询未删除的书摘和笔记并补齐引用
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 书摘或笔记标识，为空则不限制
/// - `book_id`: 书籍标识，为空则不限制
/// - `from`: 起始发生时间，包含该时刻；为空则不限制
/// - `to`: 结束发生时间，不包含该时刻；为空则不限制
///
/// # 返回值
/// 返回按发生时间倒序排列，并带上成员、标签和地点标识的记录
///
/// # 错误
/// 查询或读取引用失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = notes(conn, id, book_id, from, to)?;
/// ```
fn notes(
  conn: &Connection,
  id: Option<&str>,
  book_id: Option<&str>,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<BookNote>, LibraryError> {
  let sql = format!(
    "SELECT {NOTE_COLUMNS} FROM {NOTE_TABLE}
     WHERE deleted_at IS NULL
       AND (?1 IS NULL OR id = ?1)
       AND (?2 IS NULL OR book_id = ?2)
       AND (?3 IS NULL OR occurred_at >= ?3)
       AND (?4 IS NULL OR occurred_at < ?4)
     ORDER BY occurred_at DESC"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("book note list", e))?;
  let rows = stmt.query_map(params![id, book_id, from, to], map_note).map_err(|e| db_fail("book note list", e))?;
  let mut items = rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("book note list", e))?;
  for item in &mut items {
    let owner = Owner::BookNote.as_str();
    item.member_ids = crate::member::list_ids(conn, owner, &item.id).map_err(|_| LibraryError::Internal)?;
    item.tag_ids = crate::tag::list_ids(conn, owner, &item.id).map_err(|_| LibraryError::Internal)?;
    item.place_ids = crate::place::list_ids(conn, owner, &item.id).map_err(|_| LibraryError::Internal)?;
  }
  Ok(items)
}

/// 列出书籍下未删除书摘和笔记的标识
///
/// # 参数
/// - `conn`: 数据库连接
/// - `book_id`: 书籍标识
///
/// # 返回值
/// 返回书摘和笔记标识
///
/// # 错误
/// 查询失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let ids = note_ids(conn, book_id)?;
/// ```
fn note_ids(conn: &Connection, book_id: &str) -> Result<Vec<String>, LibraryError> {
  let mut stmt = conn
    .prepare("SELECT id FROM book_note WHERE book_id = ?1 AND deleted_at IS NULL")
    .map_err(|e| db_fail("book note ids", e))?;
  let rows = stmt.query_map([book_id], |row| row.get(0)).map_err(|e| db_fail("book note ids", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("book note ids", e))
}

/// 替换书摘或笔记上的成员、标签和地点引用
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 书摘或笔记标识
/// - `input`: 含引用标识的输入
///
/// # 返回值
/// 替换完成时返回空成功
///
/// # 错误
/// 引用不存在时返回引用不存在；其他失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// replace_refs(conn, id, input)?;
/// ```
fn replace_refs(conn: &Connection, id: &str, input: &BookNoteInput) -> Result<(), LibraryError> {
  let owner = Owner::BookNote.as_str();
  crate::member::replace(conn, owner, id, &input.member_ids).map_err(ref_err)?;
  crate::tag::replace(conn, owner, id, &input.tag_ids).map_err(ref_err)?;
  crate::place::replace(conn, owner, id, &input.place_ids).map_err(ref_err)?;
  Ok(())
}

/// 清除书摘或笔记上的成员、标签、地点、媒体和手记关联
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 书摘或笔记标识
///
/// # 返回值
/// 清除完成时返回空成功
///
/// # 错误
/// 任一清除失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// clear_note(conn, id)?;
/// ```
fn clear_note(conn: &Connection, id: &str) -> Result<(), LibraryError> {
  let owner = Owner::BookNote.as_str();
  crate::member::clear(conn, owner, id).map_err(|_| LibraryError::Internal)?;
  crate::tag::clear(conn, owner, id).map_err(|_| LibraryError::Internal)?;
  crate::place::clear(conn, owner, id).map_err(|_| LibraryError::Internal)?;
  crate::media::clear(conn, owner, id).map_err(|_| LibraryError::Internal)?;
  crate::journal::clear_note(conn, id).map_err(|_| LibraryError::Internal)?;
  Ok(())
}

/// 刷新未删除书籍的更新时间
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 书籍标识
/// - `now`: 新的更新时间
///
/// # 返回值
/// 刷新成功时返回空成功
///
/// # 错误
/// 书籍不存在时返回不存在；更新失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// touch_book(conn, id, now)?;
/// ```
fn touch_book(conn: &Connection, id: &str, now: i64) -> Result<(), LibraryError> {
  let n = conn
    .execute(
      "UPDATE book SET updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
      params![now, id],
    )
    .map_err(|e| db_fail("touch book", e))?;
  if n == 0 {
    return Err(LibraryError::NotFound);
  }
  Ok(())
}

/// 把引用失败转成书库错误
///
/// # 参数
/// - `err`: 待转换的错误
///
/// # 返回值
/// 错误文本为「引用不存在」时返回引用不存在，否则返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let mapped = ref_err(err);
/// ```
fn ref_err<E: std::fmt::Display>(err: E) -> LibraryError {
  if err.to_string() == "引用不存在" { LibraryError::ReferencedMissing } else { LibraryError::Internal }
}
