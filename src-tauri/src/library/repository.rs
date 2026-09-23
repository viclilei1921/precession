//! 书和书摘位置。书摘本身是一条 `record`。

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::{BOOK_QUOTE_TABLE, BOOK_TABLE, SCHEMA_VERSION, SCHEMA_VERSION_KEY};
use super::dto::{Book, BookWrite, QuoteItem, QuoteWrite};
use super::error::LibraryError;
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

fn db_fail(context: &str, err: rusqlite::Error) -> LibraryError {
  tauri_plugin_log::log::error!("{context}: {err}");
  LibraryError::Internal
}

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

fn map_quote(row: &rusqlite::Row<'_>) -> rusqlite::Result<QuoteItem> {
  Ok(QuoteItem {
    id: row.get(0)?,
    book_id: row.get(1)?,
    r#type: row.get(2)?,
    occurred_at: row.get(3)?,
    title: row.get(4)?,
    body: row.get(5)?,
    chapter: row.get(6)?,
    location: row.get(7)?,
    created_at: row.get(8)?,
    updated_at: row.get(9)?,
  })
}

const BOOK_COLUMNS: &str =
  "id, title, author, cover_path, status, progress, rating, started_at, finished_at, created_at, updated_at";

pub(super) fn ensure_schema(conn: &Connection) -> Result<(), LibraryError> {
  let current = schema_version(conn)?;
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

         CREATE TABLE IF NOT EXISTS book_quote (
           record_id TEXT PRIMARY KEY NOT NULL,
           book_id TEXT NOT NULL,
           chapter TEXT NOT NULL DEFAULT '',
           location TEXT NOT NULL DEFAULT ''
         );
         CREATE INDEX IF NOT EXISTS book_quote_book ON book_quote(book_id);",
      )
      .map_err(|e| db_fail("library migrate v1", e))?;
  }
  if current < SCHEMA_VERSION {
    write_schema_version(conn)?;
  }
  Ok(())
}

fn schema_version(conn: &Connection) -> Result<i64, LibraryError> {
  let current: Option<String> = conn
    .query_row("SELECT value FROM db_meta WHERE key = ?1", [SCHEMA_VERSION_KEY], |row| {
      row.get(0)
    })
    .optional()
    .map_err(|e| db_fail("library schema_version", e))?;
  Ok(current.and_then(|s| s.parse().ok()).unwrap_or(0))
}

fn write_schema_version(conn: &Connection) -> Result<(), LibraryError> {
  let version = SCHEMA_VERSION.to_string();
  conn
    .execute(
      "INSERT INTO db_meta (key, value) VALUES (?1, ?2)
       ON CONFLICT(key) DO UPDATE SET value = excluded.value",
      params![SCHEMA_VERSION_KEY, version],
    )
    .map_err(|e| db_fail("library write schema_version", e))?;
  Ok(())
}

pub(super) fn book_list(conn: &Connection) -> Result<Vec<Book>, LibraryError> {
  let sql = format!("SELECT {BOOK_COLUMNS} FROM {BOOK_TABLE} WHERE deleted_at IS NULL ORDER BY updated_at DESC");
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("book list prepare", e))?;
  let rows = stmt.query_map([], map_book).map_err(|e| db_fail("book list query", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("book list collect", e))
}

pub(super) fn book_get(conn: &Connection, id: &str) -> Result<Book, LibraryError> {
  let sql = format!("SELECT {BOOK_COLUMNS} FROM {BOOK_TABLE} WHERE id = ?1 AND deleted_at IS NULL");
  conn.query_row(&sql, [id], map_book).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => LibraryError::RecordNotFound,
    other => db_fail("book get", other),
  })
}

pub(super) fn book_create(conn: &Connection, input: &BookWrite) -> Result<Book, LibraryError> {
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

pub(super) fn book_update(conn: &Connection, id: &str, input: &BookWrite) -> Result<Book, LibraryError> {
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
    return Err(LibraryError::RecordNotFound);
  }
  book_get(conn, id)
}

pub(super) fn book_delete(conn: &Connection, id: &str) -> Result<(), LibraryError> {
  let now = now_unix_ms();
  let sql = format!("UPDATE {BOOK_TABLE} SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL");
  let n = conn.execute(&sql, params![now, id]).map_err(|e| db_fail("book delete", e))?;
  if n == 0 {
    return Err(LibraryError::RecordNotFound);
  }
  Ok(())
}

pub(super) fn quote_list(conn: &Connection, book_id: &str) -> Result<Vec<QuoteItem>, LibraryError> {
  let sql = format!(
    "SELECT r.id, q.book_id, r.type, r.occurred_at, r.title, r.body, q.chapter, q.location, r.created_at, r.updated_at
     FROM {BOOK_QUOTE_TABLE} q
     JOIN record r ON r.id = q.record_id
     WHERE q.book_id = ?1 AND r.deleted_at IS NULL
     ORDER BY r.occurred_at DESC"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("quote list prepare", e))?;
  let rows = stmt.query_map([book_id], map_quote).map_err(|e| db_fail("quote list query", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("quote list collect", e))
}

pub(super) fn quote_get(conn: &Connection, id: &str) -> Result<QuoteItem, LibraryError> {
  let sql = format!(
    "SELECT r.id, q.book_id, r.type, r.occurred_at, r.title, r.body, q.chapter, q.location, r.created_at, r.updated_at
     FROM {BOOK_QUOTE_TABLE} q
     JOIN record r ON r.id = q.record_id
     WHERE r.id = ?1 AND r.deleted_at IS NULL"
  );
  conn.query_row(&sql, [id], map_quote).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => LibraryError::RecordNotFound,
    other => db_fail("quote get", other),
  })
}

pub(super) fn quote_create(conn: &Connection, input: &QuoteWrite) -> Result<QuoteItem, LibraryError> {
  book_get(conn, &input.book_id)?;
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("quote create tx", e))?;
  tx.execute(
    "INSERT INTO record (id, type, occurred_at, title, body, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
    params![id, input.r#type, input.occurred_at, input.title, input.body, now],
  )
  .map_err(|e| db_fail("quote create record", e))?;
  let sql = format!("INSERT INTO {BOOK_QUOTE_TABLE} (record_id, book_id, chapter, location) VALUES (?1, ?2, ?3, ?4)");
  tx.execute(&sql, params![id, input.book_id, input.chapter, input.location])
    .map_err(|e| db_fail("quote create ref", e))?;
  touch_book(&tx, &input.book_id, now)?;
  tx.commit().map_err(|e| db_fail("quote create commit", e))?;
  quote_get(conn, &id)
}

pub(super) fn quote_update(conn: &Connection, id: &str, input: &QuoteWrite) -> Result<QuoteItem, LibraryError> {
  book_get(conn, &input.book_id)?;
  let previous = quote_get(conn, id)?;
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("quote update tx", e))?;
  let n = tx
    .execute(
      "UPDATE record SET type = ?1, occurred_at = ?2, title = ?3, body = ?4, updated_at = ?5
       WHERE id = ?6 AND deleted_at IS NULL",
      params![input.r#type, input.occurred_at, input.title, input.body, now, id],
    )
    .map_err(|e| db_fail("quote update record", e))?;
  if n == 0 {
    return Err(LibraryError::RecordNotFound);
  }
  let sql = format!("UPDATE {BOOK_QUOTE_TABLE} SET book_id = ?1, chapter = ?2, location = ?3 WHERE record_id = ?4");
  tx.execute(&sql, params![input.book_id, input.chapter, input.location, id])
    .map_err(|e| db_fail("quote update ref", e))?;
  touch_book(&tx, &input.book_id, now)?;
  if previous.book_id != input.book_id {
    touch_book(&tx, &previous.book_id, now)?;
  }
  tx.commit().map_err(|e| db_fail("quote update commit", e))?;
  quote_get(conn, id)
}

pub(super) fn quote_delete(conn: &Connection, id: &str) -> Result<(), LibraryError> {
  let quote = quote_get(conn, id)?;
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("quote delete tx", e))?;
  tx.execute(
    "UPDATE record SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
    params![now, id],
  )
  .map_err(|e| db_fail("quote delete record", e))?;
  let sql = format!("DELETE FROM {BOOK_QUOTE_TABLE} WHERE record_id = ?1");
  tx.execute(&sql, [id]).map_err(|e| db_fail("quote delete ref", e))?;
  for statement in [
    "DELETE FROM media WHERE record_id = ?1",
    "DELETE FROM record_member WHERE record_id = ?1",
    "DELETE FROM record_tag WHERE record_id = ?1",
    "DELETE FROM record_place WHERE record_id = ?1",
  ] {
    tx.execute(statement, [id]).map_err(|e| db_fail("quote clear", e))?;
  }
  tx.execute("DELETE FROM record_link WHERE from_id = ?1 OR to_id = ?1", [id])
    .map_err(|e| db_fail("quote clear links", e))?;
  touch_book(&tx, &quote.book_id, now)?;
  tx.commit().map_err(|e| db_fail("quote delete commit", e))?;
  Ok(())
}

fn touch_book(conn: &Connection, id: &str, now: i64) -> Result<(), LibraryError> {
  let sql = format!("UPDATE {BOOK_TABLE} SET updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL");
  let n = conn.execute(&sql, params![now, id]).map_err(|e| db_fail("touch book", e))?;
  if n == 0 {
    return Err(LibraryError::RecordNotFound);
  }
  Ok(())
}
